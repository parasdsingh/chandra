//! Shared application state.

use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

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
}

impl AppState {
    pub fn new(config_dir: PathBuf, ephemeris_dir: &std::path::Path) -> Result<Self> {
        let settings = Settings::load(&config_dir)?;
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
        })
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
            offset,
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
    /// Chandra is never included: the permanent moon item is Chandra's item, so
    /// listing it here would put two moons in the menu bar
    /// (`docs/DECISIONS.md` D-019).
    pub fn tray_subjects(&self) -> Vec<Graha> {
        let chosen = self.settings().tray.subjects;

        // Always in the canonical order, never the order they were switched on.
        // The menu bar is a row the eye learns the shape of: a graha that jumps
        // position because another was toggled off and on again makes the row
        // unlearnable, and the settings list the choices are made in is in this
        // order too.
        Graha::ALL
            .into_iter()
            .filter(|graha| *graha != Graha::Chandra && chosen.contains(graha))
            .collect()
    }

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
        let previous = self.settings();

        // The almanac rejects a zone the tz database does not know, and it is
        // the only rejection either mutation below can make on the document's
        // own contents. Asking first is what lets the file be written knowing
        // both mutations will be accepted.
        let resolved = location::resolve_offline(&next);
        jiff::tz::TimeZone::get(&resolved.zone)
            .map_err(|_| AppError::Settings(format!("unknown time zone {}", resolved.zone)))?;

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

        let tray_changed = next.tray != previous.tray;
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
        let mut settings = self.settings();
        // A manual place is authoritative and is never overridden (D-007), so a
        // device fix arriving while one is set is discarded. But only where one
        // is actually set: `mode: manual` with no place is not a choice, it is
        // the state the location gate exists to end, and refusing there made
        // "Use this Mac" a silent no-op that the gate then reported as a
        // refusal.
        if settings.location.mode == crate::settings::LocationMode::Manual
            && settings.location.place.is_some()
        {
            return Ok(());
        }

        let zone = location::from_time_zone();
        let label = chandra_geo::nearest_place(latitude, longitude)
            .map(|place| place.city.clone())
            .unwrap_or_else(|| zone.label.clone());

        settings.location.place = Some(crate::settings::PlaceSetting {
            label,
            zone: zone.zone,
            latitude,
            longitude,
            // Passed through as it arrived. `Some` only where CoreLocation
            // reported the vertical fix as valid; on a Mac with no GPS it does
            // not, and `None` is then the honest answer rather than the 0.0 the
            // framework hands back anyway.
            elevation,
        });

        self.apply(settings).map(|_| ())
    }
}

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

    /// A settings document the engine must not adopt unless it was written.
    ///
    /// The engine is a process-wide singleton, so this is the only test in the
    /// crate that may build an `AppState`.
    #[test]
    fn a_refused_save_leaves_the_engine_on_the_settings_that_are_on_disk() {
        let config_dir = std::env::temp_dir().join("chandra-apply-test");
        let _ = fs::remove_dir_all(&config_dir);
        let _ = fs::remove_file(&config_dir);
        fs::create_dir_all(&config_dir).expect("config dir");

        let state = AppState::new(config_dir.clone(), &ephemeris_dir()).expect("state");
        let before = state.settings();
        assert_eq!(before.sidereal.ayanamsa, Ayanamsa::Lahiri);

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
    #[test]
    fn the_row_reads_in_canonical_order_whatever_order_it_was_built_in() {
        let switched_on_backwards = [Graha::Shani, Graha::Chandra, Graha::Mangala, Graha::Surya];

        let ordered: Vec<Graha> = Graha::ALL
            .into_iter()
            .filter(|graha| *graha != Graha::Chandra && switched_on_backwards.contains(graha))
            .collect();

        assert_eq!(
            ordered,
            vec![Graha::Surya, Graha::Mangala, Graha::Shani],
            "canonical order, and never the moon"
        );

        // The same set switched on in a different order gives the same row.
        let switched_on_forwards = [Graha::Surya, Graha::Mangala, Graha::Shani];
        let again: Vec<Graha> = Graha::ALL
            .into_iter()
            .filter(|graha| *graha != Graha::Chandra && switched_on_forwards.contains(graha))
            .collect();
        assert_eq!(ordered, again);
    }
}
