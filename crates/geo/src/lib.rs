//! Offline location resolution.
//!
//! The whole city list is the tz database's own `zone.tab`, embedded at compile
//! time. That gives roughly 450 places, each already paired with the IANA zone
//! that governs it, with no network call, no location permission and no
//! third-party dataset to keep in sync. It is also the only list where the
//! coordinate and the timezone are guaranteed to agree with each other, which is
//! what rise and set times depend on.
//!
//! Both files are in the public domain.

mod parse;
mod search;

pub use parse::{places, Place};
pub use search::{nearest_place, place_for_zone, search};

/// Angular distance in kilometres, used to pick the closest known place to a
/// pair of coordinates.
pub(crate) fn distance_km(lat_a: f64, lon_a: f64, lat_b: f64, lon_b: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let (lat_a, lon_a) = (lat_a.to_radians(), lon_a.to_radians());
    let (lat_b, lon_b) = (lat_b.to_radians(), lon_b.to_radians());

    // Haversine rather than the spherical law of cosines: the latter loses
    // precision for nearby points, which is exactly the case being resolved.
    let d_lat = lat_b - lat_a;
    let d_lon = lon_b - lon_a;
    let h = (d_lat / 2.0).sin().powi(2) + lat_a.cos() * lat_b.cos() * (d_lon / 2.0).sin().powi(2);

    2.0 * EARTH_RADIUS_KM * h.sqrt().asin()
}
