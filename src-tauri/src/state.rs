//! Shared application state.

use std::path::PathBuf;
use std::sync::RwLock;

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
        self.settings()
            .tray
            .subjects
            .into_iter()
            .filter(|&graha| graha != Graha::Chandra)
            .collect()
    }

    /// Applies new settings, propagating whatever changed into the almanac.
    ///
    /// Returns whether the tray needs rebuilding, so the caller does not have to
    /// re-derive it by comparing settings itself.
    pub fn apply(&self, next: Settings) -> Result<Applied> {
        let previous = self.settings();

        if next.sidereal != previous.sidereal {
            self.almanac.set_sidereal(next.sidereal.into())?;
        }

        let resolved = location::resolve_offline(&next);
        let location_changed = resolved != self.location();
        if location_changed {
            self.almanac.set_location(resolved.to_location())?;
            *self.location.write().expect("location lock") = resolved;
        }

        next.save(&self.config_dir)?;
        let tray_changed = next.tray != previous.tray;
        *self.settings.write().expect("settings lock") = next;

        Ok(Applied {
            tray_changed,
            icons_changed: tray_changed || location_changed,
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
        elevation: f64,
    ) -> Result<()> {
        let mut settings = self.settings();
        if settings.location.mode == crate::settings::LocationMode::Manual {
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
            elevation,
        });

        self.apply(settings).map(|_| ())
    }
}

pub struct Applied {
    /// The set of tray items changed; add or remove them.
    pub tray_changed: bool,
    /// Existing tray icons need redrawing, for instance because the hemisphere
    /// changed and the moon disc must mirror.
    pub icons_changed: bool,
}
