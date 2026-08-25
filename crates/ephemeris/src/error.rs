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
}
