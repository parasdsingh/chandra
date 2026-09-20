pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ephemeris(#[from] chandra_ephemeris::Error),

    #[error("{year}-{month}-{day} is not a date")]
    InvalidDate { year: i16, month: i8, day: i8 },

    /// A failure inside the tz database or a local time that does not exist.
    ///
    /// Genuinely about a zone. It used to be the crate's catch-all as well -
    /// six sites raised it for a reconfiguration race and for out-of-range
    /// table indices - and `AppError::from` routes anything it does not
    /// recognise to `Engine`, so a chart that lost a generation race reached
    /// the panel as `ephemeris: time zone: the configuration kept changing
    /// while the chart was being read`, naming two subsystems that had nothing
    /// to do with it.
    #[error("time zone: {0}")]
    TimeZone(String),

    /// A reading was abandoned because the engine was reconfigured under it,
    /// three times running.
    ///
    /// Not a fault. `chakra` and `now` re-read on a generation change so they
    /// never return a value assembled half from one configuration and half from
    /// another, and they give up rather than spin if the settings keep moving.
    /// Asking again is the whole remedy, which is why it has its own code.
    #[error("the configuration kept changing while the {0} was being read")]
    Reconfiguring(&'static str),

    /// A subject asked of the view that does not serve it.
    ///
    /// A programming error, not a user-reachable one: `graha_month` refuses the
    /// Moon, which has its own view and different events.
    #[error("{what} is served by {use_instead}")]
    WrongView {
        what: &'static str,
        use_instead: &'static str,
    },

    /// An index outside the range of the table it names.
    ///
    /// Every caller clamps before it gets here, so this is a guard rather than
    /// a path: it exists so that a future caller which does not clamp fails
    /// loudly instead of naming the wrong tithi.
    #[error("{what} index {index} is out of range")]
    OutOfRange { what: &'static str, index: u8 },

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
