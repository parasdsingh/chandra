use crate::distance_km;
use crate::parse::{cities, zones, Place};

/// Exact zone lookup.
///
/// The zone table, not the city list: this is asked which place stands for a
/// zone, and only that table is guaranteed to have exactly one answer per zone.
pub fn place_for_zone(zone: &str) -> Option<&'static Place> {
    zones().iter().find(|place| place.zone == zone)
}

/// The known place closest to a pair of coordinates.
///
/// This is a **labelling aid only**. It must never be used to derive a timezone:
/// a zone boundary is political and cannot be recovered from a point, however
/// close the nearest city is. The timezone comes from the operating system,
/// which knows the answer, and only the coordinates come from the location
/// service.
///
/// It searches the city list now. Against the zone table it was answering with
/// the nearest *zone's* representative city, which is a different question and
/// gave a famously bad answer: Bengaluru is 700 km from Asia/Colombo and 1560 km
/// from Asia/Kolkata, so CoreLocation reporting Bengaluru was labelled Colombo.
/// With 34,129 cities the nearest one is nearly always the right name.
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

    cities().iter().map(|city| &city.place).min_by(|a, b| {
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

    // Population breaks the tie, largest first, before the name does. Every
    // entry that matched matched the same way, so what is left to decide is
    // which of them the user is more likely to have meant - and "york" almost
    // always means New York rather than York, Nebraska.
    let mut ranked: Vec<(u8, &'static crate::parse::City)> = cities()
        .iter()
        .filter_map(|city| rank(city, &needle).map(|score| (score, city)))
        .collect();

    ranked.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| b.1.population.cmp(&a.1.population))
            .then_with(|| a.1.place.city.cmp(&b.1.place.city))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|(_, city)| &city.place)
        .collect()
}

/// Lower is better.
fn rank(entry: &crate::parse::City, needle: &str) -> Option<u8> {
    let names = &entry.folded;
    if names.iter().any(|name| name == needle) {
        return Some(0);
    }
    if names.iter().any(|name| name.starts_with(needle)) {
        return Some(1);
    }

    // The region before the country. Someone who types a state or province name
    // is naming somewhere narrower than a country and should be answered with
    // it: `maharashtra` should not rank behind a country whose name happens to
    // contain the same letters.
    if entry
        .place
        .region
        .as_deref()
        .is_some_and(|region| region.to_lowercase().starts_with(needle))
    {
        return Some(2);
    }

    let country = entry.place.country.to_lowercase();
    if country.starts_with(needle) {
        return Some(3);
    }
    if names.iter().any(|name| name.contains(needle)) {
        return Some(4);
    }
    if entry.place.zone.to_lowercase().contains(needle) {
        return Some(5);
    }
    if country.contains(needle) {
        return Some(6);
    }
    None
}

#[cfg(test)]
mod tests {
    /// The search is fast enough to run on every keystroke.
    ///
    /// It is a linear scan of 34,129 places, and it is wired to a debounced
    /// field, so the question is whether one pass is cheap enough that the
    /// debounce is a courtesy rather than a necessity. Measured rather than
    /// assumed - the list grew by a factor of 76 in E5, and the scan that was
    /// obviously fine over 448 entries is not obviously fine over 34,129.
    ///
    /// The budget is generous on purpose: this asserts the scan has not become
    /// something that needs an index, not that it hits a particular number on a
    /// particular machine.
    #[test]
    fn one_search_is_well_inside_a_keystroke() {
        use std::time::Instant;

        // Warm the list, which is parsed once on first use. That cost is real
        // but it is paid at the first keystroke, not at every one.
        let _ = super::search("a", 1);

        let queries = ["b", "ba", "ban", "bang", "bangal", "new", "york", "z"];
        let started = Instant::now();
        for query in queries {
            let _ = super::search(query, 20);
        }
        let each = started.elapsed() / queries.len() as u32;

        assert!(
            each.as_millis() < 50,
            "one search took {each:?}, which is long enough to be felt"
        );
    }

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
    fn the_nearest_city_names_the_place_it_is_actually_at() {
        // What this used to assert was Colombo. `zone.tab` held one
        // representative city per zone, so Bengaluru - 700 km from Asia/Colombo
        // and 1560 km from Asia/Kolkata - resolved to the wrong country. The
        // city list answers the question that was actually being asked.
        let found = nearest_place(12.9716, 77.5946).unwrap();
        assert_eq!(found.city, "Bengaluru");
        assert_eq!(found.country_code, "IN");
        assert_eq!(found.region.as_deref(), Some("Karnataka"));
    }

    #[test]
    fn the_nearest_city_still_cannot_be_trusted_for_a_timezone() {
        // The rule survives the better data, and this is why. Cieszyn and Cesky
        // Tesin are one town, split down the Olza in 1920. They are 0.7 km apart
        // and in different countries and different IANA zones - so a coordinate
        // between them has a nearest city whose zone is a coin toss, and no
        // amount of precision in the city list can fix that. A zone boundary is
        // political and is not recoverable from a point.
        //
        // Found by scanning the list for the closest pair of cities in different
        // zones, rather than chosen from memory. The next four pairs are all
        // under a kilometre too.
        let poland = search("Cieszyn", 20)
            .into_iter()
            .find(|place| place.country_code == "PL")
            .expect("Cieszyn");
        // Typed the way somebody without a Czech keyboard types it, which is
        // the other half of what this test is for: the local spelling is
        // `Ceský Tesín` and an exact match on the ASCII form has to find it.
        let czechia = search("Cesky Tesin", 5)
            .into_iter()
            .find(|place| place.country_code == "CZ")
            .expect("Cesky Tesin");

        assert_ne!(poland.zone, czechia.zone);
        assert!(
            distance_km(
                poland.latitude,
                poland.longitude,
                czechia.latitude,
                czechia.longitude
            ) < 1.5,
            "the two halves of one town should be within a kilometre or so"
        );
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
