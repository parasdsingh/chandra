//! Offline location resolution.
//!
//! Two tables, because they answer two different questions.
//!
//! **The zone table** is the tz database's own `zone.tab`: one representative
//! place per IANA zone, 448 of them, each guaranteed to agree with the zone that
//! governs it. It answers "where does `Asia/Kolkata` stand for", which is
//! D-007's last step and all the app has before a location has been chosen.
//!
//! **The city table** is GeoNames' `cities15000`, trimmed by
//! `tools/geonames.sh`: 34,129 places at about 11 metres. It answers "where does
//! the user live", which the zone table never could - it is a list of zones, and
//! picking the nearest one to Bengaluru returns Colombo, 700 km away. D-007
//! promised this dataset from the first release and it had never been built
//! (E5).
//!
//! No network call and no location permission for either; both are embedded at
//! compile time. The tz files are public domain. GeoNames is CC BY 4.0, and the
//! credit that obliges is in `data/NOTICE` and in the app's About pane.

mod parse;
mod search;

pub use parse::{zones, Place};
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

    // Clamped. The haversine term reaches exactly 1 at antipodes in exact
    // arithmetic and a couple of units in the last place over it in floating
    // point, and `asin` of anything over 1 is NaN. A NaN is not a large
    // distance - it compares unordered - so it would drop that place out of the
    // comparison rather than lose to it. Searched: about one near-antipodal pair
    // in eight million produces it.
    2.0 * EARTH_RADIUS_KM * h.clamp(0.0, 1.0).sqrt().asin()
}
