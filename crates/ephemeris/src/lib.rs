//! Swiss Ephemeris boundary.
//!
//! Everything that touches the Swiss Ephemeris C library lives here, and nothing
//! above this crate is allowed to. Two properties are maintained for the layers
//! above:
//!
//! 1. **Safety.** All `unsafe` in the workspace is confined to this crate.
//! 2. **Honesty.** Swiss Ephemeris silently downgrades to the lower-precision
//!    Moshier theory when it cannot serve a date from its data files, reporting
//!    this only in a returned flag word rather than as an error. Every value this
//!    crate produces therefore carries a [`Source`] recording which theory
//!    actually computed it. See `docs/DECISIONS.md` D-006.

mod body;
mod engine;
mod error;
mod julian;
mod observer;
mod sidereal;

pub use body::Graha;
pub use engine::{Engine, Illumination, Position, Reading, RiseSet, Source};
pub use error::{Error, Result};
pub use julian::{
    from_julian_day, jd_to_unix_seconds, julian_day, unix_seconds_to_jd, JD_UNIX_EPOCH,
};
pub use observer::Observer;
pub use sidereal::{Ayanamsa, NodeType, SiderealConfig};
