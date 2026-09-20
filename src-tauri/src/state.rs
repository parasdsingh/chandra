//! Shared application state.

use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

use chandra_almanac::varga::Varga;
use chandra_almanac::{Almanac, MonthCursor};
use chandra_ephemeris::Graha;

use crate::error::{AppError, Result};
use crate::location::{self, Resolved};
use crate::settings::Settings;

pub struct AppState {
    pub almanac: Almanac,
    settings: RwLock<Settings>,
    location: RwLock<Resolved>,
    config_dir: PathBuf,
    /// Held for the whole of [`AppState::apply`].
    ///
    /// `apply` reads the current settings, decides what changed, writes the file
    /// and then swaps the value in. Two of them running at once - the settings
    /// pane and a CoreLocation answer arriving, which is the pairing that
    /// actually happens - would each decide against a document the other is
    /// about to replace, and one would overwrite the other wholesale.
    applying: Mutex<()>,
    /// Why the stored settings were not used, if they were not. See
    /// [`AppState::new`].
    settings_error: Option<String>,
}

/// How far a month cursor may be asked to travel from its anchor.
///
/// Two centuries either way. The front end re-anchors once the offset passes
/// six, so this bounds a hostile or corrupted value rather than a real one.
const MONTH_OFFSET_LIMIT: i32 = 2_400;

impl AppState {
    pub fn new(config_dir: PathBuf, ephemeris_dir: &std::path::Path) -> Result<Self> {
        // A settings file that cannot be read must not stop the app.
        //
        // This used to be `?`, which returns out of Tauri's `setup` closure -
        // and Tauri turns that into a panic. With `panic = "abort"` and an
        // accessory activation policy, the process died before it had a menu
        // bar item, so the user saw *nothing at all*: no icon, no dialog, no
        // way to discover why. Forever, on every launch, from a document a
        // downgrade or a truncated write could produce.
        //
        // The file is left exactly as found. Resetting it here would destroy
        // the only record of what the reader had chosen, which `Settings::load`
        // deliberately refuses to do. So: run on defaults, remember why, and
        // let `bootstrap` tell the reader their choices are not the ones in
        // force. Changing any setting overwrites the file, which is a
        // deliberate act rather than a silent one.
        let (settings, settings_error) = match Settings::load(&config_dir) {
            Ok(settings) => (settings, None),
            Err(error) => (Settings::default(), Some(error.to_string())),
        };
        let resolved = location::resolve_offline(&settings);

        let almanac = Almanac::new(
            ephemeris_dir,
            resolved.to_location(),
            settings.sidereal.into(),
        )
        .map_err(AppError::from)?;

        Ok(Self {
            almanac,
            settings: RwLock::new(settings),
            location: RwLock::new(resolved),
            config_dir,
            applying: Mutex::new(()),
            settings_error,
        })
    }

    /// Why the stored settings were not used, if they were not.
    pub fn settings_error(&self) -> Option<String> {
        self.settings_error.clone()
    }

    /// A month cursor in the configured month system.
    ///
    /// The system lives in settings rather than in the request, so the calendar
    /// cannot be showing lunar months while a background prefetch asks for solar
    /// ones. The opening weekday does come from the request: it is what the
    /// viewer's locale reports, which is a fact only the front end holds.
    pub fn cursor(&self, anchor_unix_ms: i64, offset: i32, first_weekday: u8) -> MonthCursor {
        MonthCursor {
            anchor_unix_ms,
            // Bounded here, where the webview's number enters.
            //
            // `first_weekday` was normalised and `offset` was not, though both
            // arrive the same way. Downstream it is added to a month number
            // (overflow: a wrap in release, a panic in debug) and, in a lunar
            // month, it is a loop count over syzygy root searches - so
            // `offset: 2_000_000` pinned a blocking thread, and `i32::MIN`
            // silently returned the anchor month because `.abs()` wrapped.
            //
            // Nothing legitimate exceeds this: the front end re-anchors at six.
            // Two centuries of months is past any scroll and far inside the
            // arithmetic.
            offset: offset.clamp(-MONTH_OFFSET_LIMIT, MONTH_OFFSET_LIMIT),
            system: self.settings().calendar.month_system,
            first_weekday: first_weekday % 7,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().expect("settings lock").clone()
    }

    pub fn location(&self) -> Resolved {
        self.location.read().expect("location lock").clone()
    }

    /// Grahas with their own menu bar item.
    ///
    /// Chandra is in here like any other calendar since D-030, which moved the
    /// permanent slot to the chart. Its item keeps the id `MOON_ID`, because it
    /// draws a phase rather than a glyph.
    pub fn tray_subjects(&self) -> Vec<Graha> {
        in_canonical_order(&self.settings().tray.subjects)
    }

    /// Divisions with their own menu bar item, in canonical order.
    ///
    /// D1 is forced in. It is the permanent chart item (D-030), and a settings
    /// file that has been hand-edited to drop it would otherwise leave the menu
    /// bar with no chart at all.
    ///
    /// Ordered like the grahas are, and for the same reason: the menu bar is a
    /// row the eye learns the shape of, and an item that jumps position because
    /// a neighbour was toggled makes the row unlearnable.
    pub fn chart_vargas(&self) -> Vec<Varga> {
        let chosen = self.settings().chart.vargas;
        Varga::ALL
            .into_iter()
            .filter(|varga| *varga == Varga::D1 || chosen.contains(varga))
            .collect()
    }
}

/// The grahas in `chosen`, always in the canonical order.
///
/// Always that order, never the order they were switched on. The menu bar is a
/// row the eye learns the shape of: a graha that jumps position because another
/// was toggled off and on again makes the row unlearnable, and the settings
/// list the choices are made in is in this order too.
///
/// A free function so the test can call the thing that ships. It used to
/// reimplement this expression and add `!= Chandra` on top - a clause the row
/// itself dropped at D-030 - so the test passed whatever `tray_subjects` did.
fn in_canonical_order(chosen: &[Graha]) -> Vec<Graha> {
    Graha::ALL
        .into_iter()
        .filter(|graha| chosen.contains(graha))
        .collect()
}

impl AppState {
    /// Applies new settings, propagating whatever changed into the almanac.
    ///
    /// Validate, persist, then mutate. Saving is the only step that can fail for
    /// a reason outside this process, and mutating the engine before it meant a
    /// refused save left every later computation running on a configuration the
    /// file and the settings view both denied.
    ///
    /// Returns whether the tray needs rebuilding, so the caller does not have to
    /// re-derive it by comparing settings itself.
    pub fn apply(&self, next: Settings) -> Result<Applied> {
        let _serialised = self.applying.lock().expect("apply lock");
        self.apply_locked(next)
    }

    /// The body of [`AppState::apply`], with the caller holding `applying`.
    ///
    /// Separate so that a caller which has to *read* the settings, change one
    /// field and write them back can hold the lock across the whole of it.
    /// `accept_device_location` is that caller, and doing its read outside the
    /// lock is exactly the interleaving `applying` was added to prevent.
    fn apply_locked(&self, next: Settings) -> Result<Applied> {
        let previous = self.settings();

        // The almanac rejects a zone the tz database does not know, and it is
        // the only rejection either mutation below can make on the document's
        // own contents. Asking first is what lets the file be written knowing
        // both mutations will be accepted.
        let resolved = location::resolve_offline(&next);
        jiff::tz::TimeZone::get(&resolved.zone)
            .map_err(|_| AppError::Settings(format!("unknown time zone {}", resolved.zone)))?;

        // The second thing either mutation can refuse, and the one that used to
        // be refused too late to matter.
        //
        // `Almanac::set_location` accepts any observer; only `rise_set` and
        // `ascendant` test it, deep inside a request. So a settings file holding
        // `latitude: 91` was saved, adopted, survived every restart - `load`
        // deliberately refuses to reset a bad file - and then failed *partly*:
        // the moon calendar drew, every graha calendar, every day view and every
        // chart returned "observer is not a position on Earth", and the reader
        // was pointed at the ephemeris rather than at their location.
        let observer = resolved.to_location().observer;
        if !observer.is_on_earth() {
            return Err(AppError::Settings(format!(
                "{}, {} is not a position on Earth",
                observer.latitude, observer.longitude
            )));
        }

        next.save(&self.config_dir)?;

        let sidereal_changed = next.sidereal != previous.sidereal;
        if sidereal_changed {
            self.almanac.set_sidereal(next.sidereal.into())?;
        }

        let location_changed = resolved != self.location();
        if location_changed {
            self.almanac.set_location(resolved.to_location())?;
            *self.location.write().expect("location lock") = resolved;
        }

        // The chart's own item counts: switching it on or off adds or removes a
        // status item exactly as a graha toggle does, and rebuilding is how a
        // status item comes and goes.
        let tray_changed = next.tray != previous.tray || next.chart.vargas != previous.chart.vargas;
        let scale_changed = next.appearance != previous.appearance;
        *self.settings.write().expect("settings lock") = next;

        Ok(Applied {
            tray_changed,
            scale_changed,
            // The ayanamsa is in here because a tray tooltip reads "{name} —
            // {rashi}", and the rashi is exactly what an ayanamsa moves.
            icons_changed: tray_changed || location_changed || sidereal_changed,
        })
    }

    /// Records coordinates that arrived from the operating system.
    ///
    /// The timezone is kept from the existing resolution rather than derived
    /// from the coordinates: `chandra_geo::nearest_place` can name a city across
    /// a border, and a wrong zone would shift every time in the app.
    pub fn accept_device_location(
        &self,
        latitude: f64,
        longitude: f64,
        elevation: Option<f64>,
    ) -> Result<()> {
        // A manual place is authoritative and is never overridden (D-007), so a
        // device fix arriving while one is set is discarded. But only where one
        // is actually set: `mode: manual` with no place is not a choice, it is
        // the state the location gate exists to end, and refusing there made
        // "Use this Mac" a silent no-op that the gate then reported as a
        // refusal.
        //
        // Asked here to avoid the work below, and asked again under the lock,
        // where the answer is the one that counts.
        if Self::manual_place_is_set(&self.settings()) {
            return Ok(());
        }

        // Before the lock, because neither of these reads the settings and the
        // second is slow: `nearest_place` is 3 ms warm and 75 ms on the first
        // call of a session, while the 34,129-row table parses.
        let zone = location::from_time_zone();
        let label = chandra_geo::nearest_place(latitude, longitude)
            .map(|place| place.city.clone())
            .unwrap_or_else(|| zone.label.clone());
        let place = crate::settings::PlaceSetting {
            label,
            zone: zone.zone,
            latitude,
            longitude,
            // Passed through as it arrived. `Some` only where CoreLocation
            // reported the vertical fix as valid; on a Mac with no GPS it does
            // not, and `None` is then the honest answer rather than the 0.0 the
            // framework hands back anyway.
            elevation,
        };

        // Read, change and write, all inside `applying`.
        //
        // This used to read the settings at the top of the function, spend the
        // milliseconds above outside any lock, and only then call `apply`, which
        // takes it. A settings change landing in that window was computed
        // against by `apply` and then overwritten wholesale by this stale
        // snapshot: clicking "Use this Mac" on a cold launch and changing the
        // ayanamsa while the fix was in flight reverted the ayanamsa, on disk
        // and in the engine, with nothing said. The same for the month system,
        // the chart's vargas, the panel scale and every panchanga toggle.
        let _serialised = self.applying.lock().expect("apply lock");
        let mut settings = self.settings();
        if Self::manual_place_is_set(&settings) {
            return Ok(());
        }
        settings.location.place = Some(place);
        self.apply_locked(settings).map(|_| ())
    }

    fn manual_place_is_set(settings: &Settings) -> bool {
        settings.location.mode == crate::settings::LocationMode::Manual
            && settings.location.place.is_some()
    }
}

#[derive(Debug)]
pub struct Applied {
    /// The set of tray items changed; add or remove them.
    pub tray_changed: bool,
    /// The panel is drawn at a different size; the window must follow it.
    pub scale_changed: bool,
    /// Existing tray icons need redrawing, for instance because the hemisphere
    /// changed and the moon disc must mirror.
    pub icons_changed: bool,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chandra_ephemeris::Ayanamsa;

    use super::*;
    use crate::settings::SiderealSetting;

    fn ephemeris_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/ephe")
    }

    /// Everything `apply` must hold, in one test.
    ///
    /// The engine is a process-wide singleton, so this is the only test in the
    /// crate that may build an `AppState` - which is why three separate
    /// guarantees are asserted here in sequence rather than in three tests.
    #[test]
    fn apply_refuses_what_it_cannot_honour_and_never_loses_a_concurrent_change() {
        let config_dir = std::env::temp_dir().join("chandra-apply-test");
        let _ = fs::remove_dir_all(&config_dir);
        let _ = fs::remove_file(&config_dir);
        fs::create_dir_all(&config_dir).expect("config dir");

        let state = AppState::new(config_dir.clone(), &ephemeris_dir()).expect("state");
        let before = state.settings();
        assert_eq!(before.sidereal.ayanamsa, Ayanamsa::Lahiri);

        // 1. An observer that is not on Earth is refused before it is written.
        //
        // Not after: `set_location` accepts anything, and only `rise_set` and
        // `ascendant` test the observer, so latitude 91 used to be saved,
        // adopted and kept across restarts while the moon calendar drew
        // normally and every chart said "observer is not a position on Earth".
        let off_earth = Settings {
            location: crate::settings::LocationSetting {
                mode: crate::settings::LocationMode::Manual,
                place: Some(crate::settings::PlaceSetting {
                    label: "nowhere".into(),
                    zone: "Asia/Kolkata".into(),
                    latitude: 91.0,
                    longitude: 0.0,
                    elevation: None,
                }),
                ..before.location.clone()
            },
            ..before.clone()
        };
        let refused = state
            .apply(off_earth)
            .expect_err("91 degrees north is not a latitude");
        assert!(
            refused.to_string().contains("not a position on Earth"),
            "the refusal must name the observer, not the ephemeris: {refused}"
        );
        assert_eq!(state.settings(), before, "a refused document was adopted");

        // 2. A device fix arriving mid-change does not revert the change.
        //
        // `accept_device_location` never touches the ayanamsa, so whatever the
        // settings pane last wrote must survive - in every interleaving. It did
        // not: the read happened outside `applying`, the geo lookup took 3 ms
        // warm and 75 ms cold, and the stale snapshot was then written back over
        // the choice just saved. Repeated, because a race that reproduces
        // sometimes is a race.
        for round in 0..24 {
            let raman = Settings {
                sidereal: SiderealSetting {
                    ayanamsa: Ayanamsa::Raman,
                    ..state.settings().sidereal
                },
                ..state.settings()
            };
            *state.settings.write().expect("settings lock") = Settings {
                sidereal: SiderealSetting {
                    ayanamsa: Ayanamsa::Lahiri,
                    ..raman.sidereal
                },
                ..raman.clone()
            };

            std::thread::scope(|scope| {
                scope.spawn(|| {
                    let _ = state.apply(raman.clone());
                });
                scope.spawn(|| {
                    let _ = state.accept_device_location(12.97, 77.59, None);
                });
            });

            assert_eq!(
                state.settings().sidereal.ayanamsa,
                Ayanamsa::Raman,
                "round {round}: a device fix reverted the ayanamsa that was just chosen"
            );
        }

        // 3. A settings document the engine must not adopt unless it was
        //    written.
        let before = Settings {
            sidereal: SiderealSetting {
                ayanamsa: Ayanamsa::Lahiri,
                ..state.settings().sidereal
            },
            ..state.settings()
        };
        state.apply(before.clone()).expect("back to lahiri");

        // A file where the configuration directory should be: `create_dir_all`
        // then fails, which is the first thing `save` does.
        fs::remove_dir_all(&config_dir).expect("clear");
        fs::write(&config_dir, "not a directory").expect("obstruct");

        let next = Settings {
            sidereal: SiderealSetting {
                ayanamsa: Ayanamsa::Raman,
                ..before.sidereal
            },
            ..before.clone()
        };
        assert!(state.apply(next).is_err(), "the save cannot have succeeded");

        assert_eq!(
            state.almanac.sidereal().expect("sidereal").ayanamsa,
            Ayanamsa::Lahiri,
            "the engine adopted a configuration that was never written"
        );
        assert_eq!(state.settings(), before);

        let _ = fs::remove_file(&config_dir);
    }
}

#[cfg(test)]
mod tray_order_tests {
    use super::*;

    /// The menu bar's order is the canonical one, not the toggle order.
    ///
    /// Filtering the stored list would preserve whatever order the user happened
    /// to switch things on in, so toggling one graha off and back on would move
    /// it to the end and shuffle the row the eye had learned.
    ///
    /// Calls `in_canonical_order`, which is what `tray_subjects` calls. It used
    /// to write the expression out again with `!= Chandra` added, so it asserted
    /// a rule the row had not followed since D-030 and would have stayed green
    /// through any change to the function it is named after.
    #[test]
    fn the_row_reads_in_canonical_order_whatever_order_it_was_built_in() {
        let backwards = [Graha::Shani, Graha::Chandra, Graha::Mangala, Graha::Surya];
        let ordered = in_canonical_order(&backwards);

        assert_eq!(
            ordered,
            vec![Graha::Surya, Graha::Chandra, Graha::Mangala, Graha::Shani],
            "canonical order, and the moon is in the row like any other graha"
        );

        // The same set switched on in a different order gives the same row.
        let forwards = [Graha::Surya, Graha::Chandra, Graha::Mangala, Graha::Shani];
        assert_eq!(ordered, in_canonical_order(&forwards));
    }
}
