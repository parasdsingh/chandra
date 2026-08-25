//! Domain layer for Chandra.
//!
//! Everything here is pure computation over the ephemeris: no Tauri, no UI, no
//! filesystem beyond the ephemeris data directory. That is what lets the whole
//! domain be tested on any platform without a windowing system.

mod almanac;
mod cache;

pub mod angles;
pub mod day;
pub mod error;
pub mod events;
pub mod lunar;
pub mod month;
pub mod muhurta;
pub mod panchanga;
pub mod phase;
pub mod roots;
pub mod spans;
pub mod standing;
pub mod time;
pub mod tithi;
pub mod zodiac;

pub use almanac::{Almanac, Location, MonthCursor, Snapshot, SnapshotGraha};
pub use error::{Error, Result};
