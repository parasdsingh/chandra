//! Errors as the front end sees them.
//!
//! Every failure carries a stable `code`. The design specifies one rendering per
//! code and no generic fallback (`docs/DESIGN.md` 9.3), so a new failure mode
//! must be given a code and a rendering rather than falling into a catch-all
//! that tells the user nothing.

use serde::Serialize;

/// There is deliberately no `LocationUnresolved` variant. The resolution chain
/// ends in the system timezone's coordinates, which is always available offline
/// (D-007), so "no location" cannot occur and an error code for it would be
/// unreachable.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A year, month and day that do not name a day - 30 February, month 13.
    ///
    /// Not an out-of-range date, which is what this used to be called. There is
    /// no out-of-range case: outside 1800-2399 the analytic fallback answers and
    /// the panel says so, which is what the precision note exists for. The old
    /// name described a case that cannot happen while being used for one that
    /// can, and the front end printed "Outside 1800-2399" for 30 February.
    #[error("{0} is not a date")]
    InvalidDate(String),

    #[error("a boundary time could not be resolved: {0}")]
    NoConvergence(String),

    #[error("ephemeris: {0}")]
    Engine(String),

    #[error("settings: {0}")]
    Settings(String),

    /// The reading was abandoned because the settings moved under it. Nothing
    /// is wrong and nothing needs fixing; asking again is the whole remedy,
    /// which is why it is not `Engine`.
    #[error("{0}")]
    Busy(String),
}

impl AppError {
    /// Stable identifier the front end switches on.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::InvalidDate(_) => "INVALID_DATE",
            AppError::NoConvergence(_) => "NO_CONVERGENCE",
            AppError::Engine(_) => "ENGINE",
            AppError::Settings(_) => "SETTINGS",
            AppError::Busy(_) => "BUSY",
        }
    }
}

impl From<chandra_almanac::Error> for AppError {
    fn from(error: chandra_almanac::Error) -> Self {
        use chandra_almanac::Error as E;
        match error {
            E::InvalidDate { year, month, day } => {
                AppError::InvalidDate(format!("{year}-{month:02}-{day:02}"))
            }
            E::NoCrossing { what, graha, .. } => {
                AppError::NoConvergence(format!("{what} for {graha}"))
            }
            E::UnknownTimeZone(zone) => AppError::Settings(format!("unknown time zone {zone}")),
            // Its own code, because it is the one failure here that is not a
            // fault and has a remedy the reader can apply. Routed through the
            // catch-all it reached the panel as "ephemeris: time zone: the
            // configuration kept changing while the chart was being read",
            // under the code ENGINE - two subsystems named that had nothing to
            // do with it, and no hint that asking again would work.
            E::Reconfiguring(_) => AppError::Busy(error.to_string()),
            other => AppError::Engine(other.to_string()),
        }
    }
}

impl From<chandra_ephemeris::Error> for AppError {
    fn from(error: chandra_ephemeris::Error) -> Self {
        AppError::Engine(error.to_string())
    }
}

/// Wire form. Tauri requires command errors to be serialisable, and a bare
/// string would lose the code the front end needs.
#[derive(Debug, Serialize)]
pub struct WireError {
    pub code: &'static str,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        WireError {
            code: self.code(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
