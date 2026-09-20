use serde::{Deserialize, Serialize};

/// A position on Earth, used only for rise and set times.
///
/// Deliberately not used for graha longitudes: panchanga is computed
/// geocentrically by tradition, and lunar parallax of up to ~1 degree would
/// otherwise shift every nakshatra boundary time (`docs/DECISIONS.md` D-004).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Observer {
    /// Degrees north, negative south.
    pub latitude: f64,
    /// Degrees east, negative west.
    pub longitude: f64,
    /// Metres above sea level.
    pub elevation: f64,
}

impl Observer {
    /// The heights `swe_rise_trans` accepts, and therefore the only heights this
    /// app can answer at.
    ///
    /// Public because three other places need the same bound and each had
    /// written it out: the city table filters GeoNames' `-9999` sentinel against
    /// it at parse, and the settings layer clamps the user's own correction to
    /// it. Outside it there is no sunrise, no moonrise and no graha rise until
    /// the location changes, which is too total a failure to let a typed number
    /// cause.
    pub const ELEVATION_MIN: f64 = -500.0;
    pub const ELEVATION_MAX: f64 = 25_000.0;

    pub const fn new(latitude: f64, longitude: f64, elevation: f64) -> Self {
        Self {
            latitude,
            longitude,
            elevation,
        }
    }

    /// Whether this is a position on Earth at all.
    ///
    /// Latitude within the poles, longitude within a turn, elevation inside the
    /// range `swe_rise_trans` itself accepts, and every one of them finite.
    ///
    /// `CLLocationCoordinate2DInvalid` is `(-180, -180)`, a hand-edited settings
    /// file can hold anything, and neither is refused by Swiss Ephemeris - so
    /// without this the app answers such a question instead of declining it.
    pub fn is_on_earth(&self) -> bool {
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && self.elevation.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=360.0).contains(&self.longitude)
            && (Self::ELEVATION_MIN..=Self::ELEVATION_MAX).contains(&self.elevation)
    }

    /// Swiss Ephemeris expects `[longitude, latitude, elevation]` in that order,
    /// which is the reverse of how coordinates are normally written and spoken.
    /// Keeping the swap in one named place stops it being re-introduced as a bug.
    pub(crate) fn as_se_geopos(&self) -> [f64; 3] {
        [self.longitude, self.latitude, self.elevation]
    }
}
