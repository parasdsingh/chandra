//! Observer location.
//!
//! Resolution follows the chain in `docs/DECISIONS.md` D-007. The first step
//! that produces an answer wins, and every step below the first is offline, so a
//! usable location always exists and the first panel open is never blocked:
//!
//! 1. a manual override the user set;
//! 2. CoreLocation, if macOS grants it;
//! 3. the system timezone's representative coordinates from `zone.tab`.
//!
//! The timezone always comes from the operating system, never from the
//! coordinates. `chandra_geo::nearest_place` can cross a border - Bengaluru
//! resolves to Asia/Colombo - so deriving a zone from coordinates would silently
//! shift every displayed time.

use chandra_almanac::Location;
use chandra_ephemeris::Observer;

use crate::settings::{LocationMode, PlaceSetting, Settings};

/// Which step of the chain produced the location in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Manual,
    CoreLocation,
    /// Coordinates of the system timezone's representative city.
    TimeZone,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Resolved {
    pub label: String,
    pub zone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub provenance: Provenance,
}

impl Resolved {
    pub fn to_location(&self) -> Location {
        Location {
            observer: Observer::new(self.latitude, self.longitude, self.elevation),
            zone_name: self.zone.clone(),
        }
    }
}

/// Resolves without consulting CoreLocation.
///
/// This is what runs at startup: it cannot fail, needs no permission and touches
/// no network, so the app is usable in its first frame. A CoreLocation answer,
/// if one ever arrives, refines it afterwards.
pub fn resolve_offline(settings: &Settings) -> Resolved {
    let mut resolved = resolve_place(settings);

    // The user's own correction, on top of whichever step answered. No step
    // supplies one: `zone.tab` has no elevation column and CoreLocation's
    // vertical fix is poor, which is why the setting exists at all.
    if let Some(elevation) = settings.location.elevation {
        resolved.elevation = elevation;
    }
    resolved
}

fn resolve_place(settings: &Settings) -> Resolved {
    if settings.location.mode == LocationMode::Manual {
        if let Some(place) = &settings.location.place {
            return from_place(place, Provenance::Manual);
        }
    }

    // A cached automatic result is preferred over the timezone centroid: it came
    // from the device and is more precise than a city centre.
    if let Some(place) = &settings.location.place {
        return from_place(place, Provenance::CoreLocation);
    }

    from_time_zone()
}

fn from_place(place: &PlaceSetting, provenance: Provenance) -> Resolved {
    Resolved {
        label: place.label.clone(),
        zone: place.zone.clone(),
        latitude: place.latitude,
        longitude: place.longitude,
        elevation: place.elevation,
        provenance,
    }
}

/// The system timezone's representative city.
///
/// Falls back to UTC at Greenwich if the machine reports a zone that is not in
/// the table, which happens only for a hand-edited or fixed-offset zone.
pub fn from_time_zone() -> Resolved {
    for_zone(&system_zone_name())
}

/// The representative city for a named zone, or Greenwich if the table has none.
///
/// Split from [`from_time_zone`] so the fallback can be asserted: reading the
/// machine's own zone made the Greenwich branch untestable, and the test that
/// was meant to cover it built a `Resolved` by hand and compared it with itself.
fn for_zone(zone_name: &str) -> Resolved {
    match chandra_geo::place_for_zone(zone_name) {
        Some(place) => Resolved {
            label: place.city.clone(),
            zone: place.zone.clone(),
            latitude: place.latitude,
            longitude: place.longitude,
            // zone.tab carries no elevation. Sea level understates rise times by
            // about four minutes at 900 m, which is why the setting exists.
            elevation: 0.0,
            provenance: Provenance::TimeZone,
        },
        None => Resolved {
            label: "Greenwich".into(),
            zone: "UTC".into(),
            latitude: 51.4779,
            longitude: 0.0,
            elevation: 0.0,
            provenance: Provenance::TimeZone,
        },
    }
}

fn system_zone_name() -> String {
    jiff::tz::TimeZone::system()
        .iana_name()
        .unwrap_or("UTC")
        .to_string()
}

/// Outcome of asking macOS for the device's coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Located {
        latitude: f64,
        longitude: f64,
        elevation: f64,
    },
    /// The user, or a policy, said no. Not retried.
    Denied,
    /// Location services are switched off, or the framework reported a failure.
    Unavailable,
}

#[cfg(target_os = "macos")]
pub use platform::{release, request};

#[cfg(target_os = "macos")]
mod platform {
    use std::cell::RefCell;

    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
    use objc2_core_location::{
        CLAuthorizationStatus, CLLocation, CLLocationManager, CLLocationManagerDelegate,
    };
    use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol};

    use super::Outcome;

    /// Callback invoked exactly once, on the main thread.
    type Sink = Box<dyn FnMut(Outcome)>;

    struct Ivars {
        sink: RefCell<Option<Sink>>,
    }

    define_class!(
        // A plain NSObject subclass; CoreLocation only requires that the
        // delegate be created on a thread with an active run loop, which is the
        // main thread here.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "ChandraLocationDelegate"]
        #[ivars = Ivars]
        struct Delegate;

        unsafe impl NSObjectProtocol for Delegate {}

        unsafe impl CLLocationManagerDelegate for Delegate {
            #[unsafe(method(locationManager:didUpdateLocations:))]
            unsafe fn did_update_locations(
                &self,
                _manager: &CLLocationManager,
                locations: &NSArray<CLLocation>,
            ) {
                // The array is ordered oldest to newest; the last is the best
                // fix CoreLocation has.
                let Some(location) = locations.into_iter().last() else {
                    return;
                };
                let coordinate = unsafe { location.coordinate() };
                let altitude = unsafe { location.altitude() };

                self.deliver(Outcome::Located {
                    latitude: coordinate.latitude,
                    longitude: coordinate.longitude,
                    // A negative altitude below sea level is physically possible
                    // but is far more often a poor vertical fix, and a negative
                    // elevation would shift rise times the wrong way.
                    elevation: altitude.max(0.0),
                });
            }

            #[unsafe(method(locationManager:didFailWithError:))]
            unsafe fn did_fail(&self, _manager: &CLLocationManager, _error: &NSError) {
                self.deliver(Outcome::Unavailable);
            }

            #[unsafe(method(locationManagerDidChangeAuthorization:))]
            unsafe fn did_change_authorization(&self, manager: &CLLocationManager) {
                match unsafe { manager.authorizationStatus() } {
                    CLAuthorizationStatus::AuthorizedAlways
                    | CLAuthorizationStatus::AuthorizedWhenInUse => unsafe {
                        manager.requestLocation()
                    },
                    CLAuthorizationStatus::Denied | CLAuthorizationStatus::Restricted => {
                        self.deliver(Outcome::Denied)
                    }
                    // NotDetermined: the prompt is still on screen. Waiting is
                    // correct; the timeout in the caller bounds it.
                    _ => {}
                }
            }
        }
    );

    impl Delegate {
        fn new(marker: MainThreadMarker, sink: Sink) -> Retained<Self> {
            let this = Self::alloc(marker).set_ivars(Ivars {
                sink: RefCell::new(Some(sink)),
            });
            unsafe { msg_send![super(this), init] }
        }

        /// Fires the callback at most once, so a delegate that receives both a
        /// location and a later failure does not report twice.
        fn deliver(&self, outcome: Outcome) {
            if let Some(mut sink) = self.ivars().sink.borrow_mut().take() {
                sink(outcome);
            }
        }
    }

    // Keeps the manager and delegate alive for the duration of a request.
    // CoreLocation holds only a weak reference to its delegate, and a dropped
    // manager stops updating, so both must outlive the call that started them.
    // Neither type is `Send`, so they live in main-thread storage.
    thread_local! {
        static IN_FLIGHT: RefCell<Option<(Retained<CLLocationManager>, Retained<Delegate>)>> =
            const { RefCell::new(None) };
    }

    /// Asks macOS for the device's coordinates.
    ///
    /// Must be called on the main thread. `on_result` runs on the main thread
    /// too, exactly once, unless authorisation is never answered - the caller is
    /// responsible for giving up after a timeout.
    pub fn request(on_result: impl FnMut(Outcome) + 'static) {
        let Some(marker) = MainThreadMarker::new() else {
            // Called off the main thread. Reporting unavailable is the honest
            // answer; starting CoreLocation here would deliver callbacks to a
            // run loop that never runs.
            let mut on_result = on_result;
            on_result(Outcome::Unavailable);
            return;
        };

        if !unsafe { CLLocationManager::locationServicesEnabled_class() } {
            let mut on_result = on_result;
            on_result(Outcome::Unavailable);
            return;
        }

        let delegate = Delegate::new(marker, Box::new(on_result));
        let manager: Retained<CLLocationManager> = unsafe { CLLocationManager::new() };

        unsafe {
            manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }

        let status = unsafe { manager.authorizationStatus() };
        match status {
            CLAuthorizationStatus::AuthorizedAlways
            | CLAuthorizationStatus::AuthorizedWhenInUse => unsafe { manager.requestLocation() },
            CLAuthorizationStatus::NotDetermined => unsafe {
                manager.requestWhenInUseAuthorization()
            },
            _ => {
                delegate.deliver(Outcome::Denied);
                return;
            }
        }

        IN_FLIGHT.with(|slot| {
            // Replacing any previous request cancels it, which is what should
            // happen: only the newest answer is wanted.
            *slot.borrow_mut() = Some((manager, delegate));
        });
    }

    /// Releases whatever a finished request left alive.
    ///
    /// Must be called on the main thread, once the caller has stopped waiting.
    /// The pair is held for the duration of a request because CoreLocation keeps
    /// only a weak reference to its delegate and a dropped manager stops
    /// updating; held past it, an unanswered authorisation prompt kept a
    /// `CLLocationManager` and its delegate alive until the next request came,
    /// which for a user who never answers is for the life of the process.
    pub fn release() {
        IN_FLIGHT.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }
}

/// Location services exist only on macOS in this build. Every other platform
/// resolves through the timezone, which is a complete answer rather than a stub.
#[cfg(not(target_os = "macos"))]
pub fn request(mut on_result: impl FnMut(Outcome) + 'static) {
    on_result(Outcome::Unavailable);
}

/// Nothing is ever held alive off macOS, so there is nothing to release.
#[cfg(not(target_os = "macos"))]
pub fn release() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{LocationSetting, PlaceSetting};

    fn manual(place: PlaceSetting) -> Settings {
        Settings {
            location: LocationSetting {
                mode: LocationMode::Manual,
                place: Some(place),
                elevation: None,
            },
            ..Settings::default()
        }
    }

    fn bengaluru() -> PlaceSetting {
        PlaceSetting {
            label: "Bengaluru".into(),
            zone: "Asia/Kolkata".into(),
            latitude: 12.9716,
            longitude: 77.5946,
            elevation: 920.0,
        }
    }

    #[test]
    fn a_manual_override_wins() {
        let resolved = resolve_offline(&manual(bengaluru()));
        assert_eq!(resolved.provenance, Provenance::Manual);
        assert_eq!(resolved.zone, "Asia/Kolkata");
        assert_eq!(resolved.elevation, 920.0);
    }

    #[test]
    fn without_a_manual_override_the_timezone_answers() {
        let resolved = resolve_offline(&Settings::default());
        assert_eq!(resolved.provenance, Provenance::TimeZone);
        // Whatever this machine's zone is, the result must be usable.
        assert!((-90.0..=90.0).contains(&resolved.latitude));
        assert!((-180.0..=180.0).contains(&resolved.longitude));
        assert!(!resolved.zone.is_empty());
    }

    #[test]
    fn the_offline_chain_never_fails() {
        // The whole point of step 3: there is no configuration in which the app
        // has no location and cannot open.
        for mode in [LocationMode::Automatic, LocationMode::Manual] {
            let settings = Settings {
                location: LocationSetting {
                    mode,
                    place: None,
                    elevation: None,
                },
                ..Settings::default()
            };
            let resolved = resolve_offline(&settings);
            assert!(!resolved.zone.is_empty(), "mode {mode:?} produced no zone");
        }
    }

    #[test]
    fn a_cached_automatic_place_is_reused() {
        let settings = Settings {
            location: LocationSetting {
                mode: LocationMode::Automatic,
                place: Some(bengaluru()),
                elevation: None,
            },
            ..Settings::default()
        };
        let resolved = resolve_offline(&settings);
        assert_eq!(resolved.provenance, Provenance::CoreLocation);
        assert_eq!(resolved.latitude, 12.9716);
    }

    /// Setting an elevation must not turn the answer into a different one.
    ///
    /// It used to be stored by writing a whole place around it, which in
    /// automatic mode made a timezone-derived location report itself as
    /// `CoreLocation` - "from this Mac" - for a location the Mac never gave.
    #[test]
    fn an_elevation_correction_does_not_change_where_the_location_came_from() {
        let settings = Settings {
            location: LocationSetting {
                mode: LocationMode::Automatic,
                place: None,
                elevation: Some(920.0),
            },
            ..Settings::default()
        };
        let resolved = resolve_offline(&settings);

        assert_eq!(resolved.elevation, 920.0);
        assert_eq!(
            resolved.provenance,
            Provenance::TimeZone,
            "the timezone still answered; only its elevation was corrected"
        );

        let uncorrected = resolve_offline(&Settings::default());
        assert_eq!(resolved.zone, uncorrected.zone);
        assert_eq!(resolved.latitude, uncorrected.latitude);
        assert_eq!(resolved.longitude, uncorrected.longitude);
    }

    #[test]
    fn an_unknown_zone_falls_back_to_greenwich() {
        let fallback = for_zone("Not/AZone");
        assert_eq!(fallback.label, "Greenwich");
        assert_eq!(fallback.zone, "UTC");
        assert_eq!(fallback.longitude, 0.0);
        assert_eq!(fallback.provenance, Provenance::TimeZone);

        // A zone the table does know resolves to its own representative city,
        // so the fallback is a fallback rather than the only branch that runs.
        let known = for_zone("Asia/Kolkata");
        assert_eq!(known.zone, "Asia/Kolkata");
        assert_ne!(known.label, "Greenwich");
        assert_eq!(known.provenance, Provenance::TimeZone);
    }
}
