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
    pub const fn new(latitude: f64, longitude: f64, elevation: f64) -> Self {
        Self {
            latitude,
            longitude,
            elevation,
        }
    }

    /// Swiss Ephemeris expects `[longitude, latitude, elevation]` in that order,
    /// which is the reverse of how coordinates are normally written and spoken.
    /// Keeping the swap in one named place stops it being re-introduced as a bug.
    pub(crate) fn as_se_geopos(&self) -> [f64; 3] {
        [self.longitude, self.latitude, self.elevation]
    }

    /// True when the latitude is high enough that a body may stay above or below
    /// the horizon for a whole day, so a missing rise or set is expected rather
    /// than a failure.
    pub fn is_polar(&self) -> bool {
        self.latitude.abs() > 60.0
    }
}
