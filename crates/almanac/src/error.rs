pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ephemeris(#[from] chandra_ephemeris::Error),

    #[error("{year}-{month}-{day} is not a date")]
    InvalidDate { year: i16, month: i8, day: i8 },

    #[error("time zone: {0}")]
    TimeZone(String),

    #[error("unknown time zone: {0}")]
    UnknownTimeZone(String),

    /// A boundary crossing that must exist within the search window was not
    /// found. Surfaced rather than papered over: a guessed time would be
    /// indistinguishable from a real one on screen.
    #[error("no {what} found for {graha} within {window_days} days of JD {near}")]
    NoCrossing {
        what: &'static str,
        graha: &'static str,
        near: f64,
        window_days: f64,
    },
}
