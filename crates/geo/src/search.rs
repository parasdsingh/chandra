use crate::distance_km;
use crate::parse::{places, Place};

/// Exact zone lookup.
pub fn place_for_zone(zone: &str) -> Option<&'static Place> {
    places().iter().find(|place| place.zone == zone)
}

/// The known place closest to a pair of coordinates.
///
/// This is a **labelling aid only**. It must never be used to derive a timezone.
/// `zone.tab` lists one representative city per zone, so the nearest entry can
/// easily sit in another country: Bengaluru is 700 km from Asia/Colombo and
/// 1560 km from Asia/Kolkata, so this returns Colombo. The timezone comes from
/// the operating system, which knows the answer, and only the coordinates come
/// from the location service.
///
/// Never returns `None` for real coordinates: the table covers every inhabited
/// zone, so there is always a nearest entry. `None` means the coordinates were
/// not coordinates - a NaN or a value off the globe - which is a thing a
/// location service can hand over and which no place is nearest to. Without the
/// test every distance is NaN, they all compare equal under `total_cmp`, and the
/// first row of the table comes back as confidently as a real answer.
pub fn nearest_place(latitude: f64, longitude: f64) -> Option<&'static Place> {
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return None;
    }

    places().iter().min_by(|a, b| {
        distance_km(latitude, longitude, a.latitude, a.longitude).total_cmp(&distance_km(
            latitude,
            longitude,
            b.latitude,
            b.longitude,
        ))
    })
}

/// City search, ranked so the most likely match is first.
///
/// Ranking is by where the query matched rather than by string distance: a
/// prefix match on the city name is what the user almost always means, and
/// putting a mid-word match from a different continent above it would make the
/// field feel broken.
pub fn search(query: &str, limit: usize) -> Vec<&'static Place> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(u8, &'static Place)> = places()
        .iter()
        .filter_map(|place| rank(place, &needle).map(|score| (score, place)))
        .collect();

    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.city.cmp(&b.1.city)));
    ranked
        .into_iter()
        .take(limit)
        .map(|(_, place)| place)
        .collect()
}

/// Lower is better.
fn rank(place: &Place, needle: &str) -> Option<u8> {
    let city = place.city.to_lowercase();
    if city == needle {
        return Some(0);
    }
    if city.starts_with(needle) {
        return Some(1);
    }

    let country = place.country.to_lowercase();
    if country.starts_with(needle) {
        return Some(2);
    }
    if city.contains(needle) {
        return Some(3);
    }
    if place.zone.to_lowercase().contains(needle) {
        return Some(4);
    }
    if country.contains(needle) {
        return Some(5);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_city_name_ranks_first() {
        let results = search("kolkata", 10);
        assert_eq!(results[0].zone, "Asia/Kolkata");
    }

    #[test]
    fn prefix_beats_substring() {
        let results = search("lond", 10);
        assert_eq!(results[0].city, "London");
    }

    #[test]
    fn search_is_case_and_whitespace_insensitive() {
        assert_eq!(search("  ToKyO ", 5)[0].zone, "Asia/Tokyo");
    }

    #[test]
    fn country_names_are_searchable() {
        let results = search("iceland", 5);
        assert!(
            results.iter().any(|p| p.zone == "Atlantic/Reykjavik"),
            "searching a country should surface its zones"
        );
    }

    #[test]
    fn an_empty_query_returns_nothing_rather_than_everything() {
        assert!(search("", 10).is_empty());
        assert!(search("   ", 10).is_empty());
    }

    #[test]
    fn the_limit_is_respected() {
        assert!(search("a", 5).len() <= 5);
    }

    #[test]
    fn nonsense_matches_nothing() {
        assert!(search("qqzzxx", 10).is_empty());
    }

    #[test]
    fn nearest_place_finds_the_closest_entry_in_the_table() {
        let found = nearest_place(51.5074, -0.1278).unwrap();
        assert_eq!(found.zone, "Europe/London");

        let found = nearest_place(-33.8688, 151.2093).unwrap();
        assert_eq!(found.zone, "Australia/Sydney");

        let found = nearest_place(40.7128, -74.0060).unwrap();
        assert_eq!(found.zone, "America/New_York");
    }

    #[test]
    fn nearest_place_can_cross_a_border_and_must_not_set_the_timezone() {
        // zone.tab holds one representative city per zone, so a place far from
        // its own zone's representative resolves to a nearer foreign one.
        // Bengaluru is 700 km from Colombo and 1560 km from Kolkata. This is
        // asserted rather than worked around, because it is the reason the
        // timezone is taken from the operating system instead of from here.
        let found = nearest_place(12.9716, 77.5946).unwrap();
        assert_eq!(found.zone, "Asia/Colombo");
        assert_ne!(found.country_code, "IN");
    }

    #[test]
    fn zone_lookup_is_exact() {
        assert!(place_for_zone("Asia/Kolkata").is_some());
        assert!(place_for_zone("asia/kolkata").is_none());
        assert!(place_for_zone("Not/AZone").is_none());
    }

    #[test]
    fn a_coordinate_that_is_not_one_has_no_nearest_place() {
        // CoreLocation can hand over anything. Every distance from a NaN is a
        // NaN, and NaNs compare equal to each other, so the first row of the
        // table used to be returned as the answer.
        assert!(nearest_place(f64::NAN, 0.0).is_none());
        assert!(nearest_place(0.0, f64::NAN).is_none());
        assert!(nearest_place(f64::INFINITY, 0.0).is_none());
        assert!(nearest_place(91.0, 0.0).is_none());
        assert!(nearest_place(0.0, -181.0).is_none());
        assert!(nearest_place(0.0, 0.0).is_some());
    }

    #[test]
    fn antipodes_are_half_a_circumference_apart_and_not_a_nan() {
        // The haversine term rounds just over 1 for some antipodal pairs, and
        // `asin` of that is a NaN - which is not a large distance, it is no
        // distance, and it silently drops out of any comparison.
        for (lat, lon) in [
            (0.0, 0.0),
            (-87.5, -180.0),
            (45.0, 30.0),
            (12.9716, 77.5946),
        ] {
            let (other_lat, other_lon) = (-lat, if lon >= 0.0 { lon - 180.0 } else { lon + 180.0 });
            let d = distance_km(lat, lon, other_lat, other_lon);
            assert!(d.is_finite(), "({lat}, {lon}) to its antipode gave {d}");
            assert!(
                (d - 20_015.0).abs() < 1.0,
                "({lat}, {lon}) to its antipode is {d} km, not half a circumference"
            );
        }
    }

    #[test]
    fn distance_is_symmetric_and_sane() {
        // London to Paris is about 344 km.
        let d = distance_km(51.5074, -0.1278, 48.8566, 2.3522);
        assert!((d - 344.0).abs() < 10.0, "got {d} km");
        let reverse = distance_km(48.8566, 2.3522, 51.5074, -0.1278);
        assert!((d - reverse).abs() < 1e-9);
        assert_eq!(distance_km(10.0, 20.0, 10.0, 20.0), 0.0);
    }
}
