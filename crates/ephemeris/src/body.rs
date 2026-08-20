use serde::{Deserialize, Serialize};

/// The nine grahas, in traditional order.
///
/// Ketu has no Swiss Ephemeris body of its own: it is always exactly opposite
/// Rahu, so it is computed from the node and reflected by 180 degrees. That
/// derivation is handled in [`crate::Engine`] rather than exposed here, so
/// callers can treat all nine uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Graha {
    Surya,
    Chandra,
    Mangala,
    Budha,
    Guru,
    Shukra,
    Shani,
    Rahu,
    Ketu,
}

impl Graha {
    pub const ALL: [Graha; 9] = [
        Graha::Surya,
        Graha::Chandra,
        Graha::Mangala,
        Graha::Budha,
        Graha::Guru,
        Graha::Shukra,
        Graha::Shani,
        Graha::Rahu,
        Graha::Ketu,
    ];

    /// Sanskrit name, as displayed.
    pub const fn name(self) -> &'static str {
        match self {
            Graha::Surya => "Surya",
            Graha::Chandra => "Chandra",
            Graha::Mangala => "Mangala",
            Graha::Budha => "Budha",
            Graha::Guru => "Guru",
            Graha::Shukra => "Shukra",
            Graha::Shani => "Shani",
            Graha::Rahu => "Rahu",
            Graha::Ketu => "Ketu",
        }
    }

    /// Common English name, shown as a secondary label.
    pub const fn english(self) -> &'static str {
        match self {
            Graha::Surya => "Sun",
            Graha::Chandra => "Moon",
            Graha::Mangala => "Mars",
            Graha::Budha => "Mercury",
            Graha::Guru => "Jupiter",
            Graha::Shukra => "Venus",
            Graha::Shani => "Saturn",
            Graha::Rahu => "North Node",
            Graha::Ketu => "South Node",
        }
    }

    /// Stable lowercase key used in settings files and IPC payloads.
    pub const fn key(self) -> &'static str {
        match self {
            Graha::Surya => "surya",
            Graha::Chandra => "chandra",
            Graha::Mangala => "mangala",
            Graha::Budha => "budha",
            Graha::Guru => "guru",
            Graha::Shukra => "shukra",
            Graha::Shani => "shani",
            Graha::Rahu => "rahu",
            Graha::Ketu => "ketu",
        }
    }

    /// The luminaries never retrograde, and the nodes are always retrograde
    /// (mean) or nearly so (true). Used to decide whether to scan for stations
    /// at all, which removes two thirds of the event search for these bodies.
    pub const fn can_station(self) -> bool {
        matches!(
            self,
            Graha::Mangala | Graha::Budha | Graha::Guru | Graha::Shukra | Graha::Shani
        )
    }

    /// Typical time to traverse one degree, used to choose a scan step coarse
    /// enough to be cheap and fine enough that no ingress is stepped over.
    /// Values are deliberately conservative (faster than the true maximum).
    pub const fn scan_step_days(self) -> f64 {
        match self {
            // ~13.2 deg/day: a nakshatra boundary can be crossed in under a day.
            Graha::Chandra => 1.0 / 24.0,
            // ~1 deg/day, but Budha and Shukra swing either side of it.
            Graha::Surya | Graha::Budha | Graha::Shukra | Graha::Mangala => 0.25,
            Graha::Guru | Graha::Shani | Graha::Rahu | Graha::Ketu => 1.0,
        }
    }
}

/// Swiss Ephemeris body identifiers.
///
/// The `swiss-eph` crate exposes these with inconsistent integer types (some
/// `u32`, some `i32`), so they are normalised to `i32` once, here, and never
/// referenced directly anywhere else.
pub(crate) mod se_body {
    pub const SUN: i32 = 0;
    pub const MOON: i32 = 1;
    pub const MERCURY: i32 = 2;
    pub const VENUS: i32 = 3;
    pub const MARS: i32 = 4;
    pub const JUPITER: i32 = 5;
    pub const SATURN: i32 = 6;
    pub const MEAN_NODE: i32 = 10;
    pub const TRUE_NODE: i32 = 11;
}
