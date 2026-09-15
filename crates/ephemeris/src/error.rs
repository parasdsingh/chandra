use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ephemeris data directory not found: {0}")]
    EphemerisPathMissing(PathBuf),

    /// The directory exists but Swiss Ephemeris did not serve a date from it.
    /// Distinguished from a missing directory because the two are fixed
    /// differently: one is a wrong path, the other is a path to nothing.
    #[error("no ephemeris data files in {0}; every result would silently be Moshier")]
    EphemerisDataUnusable(PathBuf),

    #[error("ephemeris path is not valid UTF-8 or contains a NUL byte: {0}")]
    EphemerisPathInvalid(PathBuf),

    /// Swiss Ephemeris reported a hard failure. The message is the library's own
    /// error buffer, which is far more specific than any wording we could invent.
    #[error("swiss ephemeris: {message} (while computing {context})")]
    Calculation { context: String, message: String },

    /// Swiss Ephemeris state is process-global, so a second engine would
    /// reconfigure the first one behind its back.
    #[error("an ephemeris engine already exists in this process")]
    AlreadyConstructed,

    /// The engine mutex was poisoned by a panic in another thread. Unrecoverable,
    /// because the C library's global state is then of unknown validity.
    #[error("ephemeris engine is poisoned; a previous calculation panicked")]
    Poisoned,

    /// A position that is not on Earth.
    ///
    /// Swiss Ephemeris does not refuse these. `swe_houses` returns success for a
    /// latitude of 91, of -180 and of 1e9, giving an ascendant that looks like
    /// an answer; a NaN latitude makes `swe_rise_trans` return a rise and a set
    /// at the *same* instant, two hours after the search began, which is the
    /// origin of its internal grid rather than an event in the sky. A fabricated
    /// answer is worse than an error, so the check is here.
    #[error(
        "observer is not a position on Earth: latitude {latitude}, longitude {longitude}, \
         elevation {elevation} m"
    )]
    InvalidObserver {
        latitude: f64,
        longitude: f64,
        elevation: f64,
    },
}
