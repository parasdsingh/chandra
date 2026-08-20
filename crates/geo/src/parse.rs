use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// The tz database's zone table and country code table, embedded at build time
/// so the app never reads the host's copy. Depending on the host's `zoneinfo`
/// would make results vary with the machine's OS version.
const ZONE_TAB: &str = include_str!("../data/zone.tab");
const ISO3166_TAB: &str = include_str!("../data/iso3166.tab");

/// A place the user can pick: an IANA zone with the coordinates of its
/// representative location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    /// IANA zone name, for example `Asia/Kolkata`.
    pub zone: String,
    /// Representative city, derived from the last segment of the zone name.
    pub city: String,
    pub country: String,
    pub country_code: String,
    /// Degrees north, negative south.
    pub latitude: f64,
    /// Degrees east, negative west.
    pub longitude: f64,
}

/// Every place, parsed once and sorted by city name for stable ordering.
pub fn places() -> &'static [Place] {
    static PLACES: OnceLock<Vec<Place>> = OnceLock::new();
    PLACES.get_or_init(|| {
        let countries = parse_countries();
        let mut parsed: Vec<Place> = ZONE_TAB
            .lines()
            .filter_map(|line| parse_zone_line(line, &countries))
            .collect();
        parsed.sort_by(|a, b| a.city.cmp(&b.city).then_with(|| a.zone.cmp(&b.zone)));
        parsed
    })
}

fn parse_countries() -> Vec<(String, String)> {
    ISO3166_TAB
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (code, name) = line.split_once('\t')?;
            Some((code.trim().to_string(), name.trim().to_string()))
        })
        .collect()
}

fn parse_zone_line(line: &str, countries: &[(String, String)]) -> Option<Place> {
    if line.starts_with('#') || line.trim().is_empty() {
        return None;
    }

    let mut fields = line.split('\t');
    let country_code = fields.next()?.trim();
    let coordinates = fields.next()?.trim();
    let zone = fields.next()?.trim();

    let (latitude, longitude) = parse_iso6709(coordinates)?;

    Some(Place {
        city: city_from_zone(zone),
        zone: zone.to_string(),
        country: countries
            .iter()
            .find(|(code, _)| code == country_code)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| country_code.to_string()),
        country_code: country_code.to_string(),
        latitude,
        longitude,
    })
}

/// The last segment of a zone name, with the tz database's underscores restored
/// to spaces: `America/Argentina/Rio_Gallegos` becomes `Rio Gallegos`.
fn city_from_zone(zone: &str) -> String {
    zone.rsplit('/').next().unwrap_or(zone).replace('_', " ")
}

/// Parses the tz database's ISO 6709 coordinate form.
///
/// Two widths occur: `+DDMM+DDDMM` and `+DDMMSS+DDDMMSS`. The sign is always
/// present, which is what makes the fields splittable without a separator.
fn parse_iso6709(text: &str) -> Option<(f64, f64)> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || !matches!(bytes[0], b'+' | b'-') {
        return None;
    }

    // The longitude begins at the second sign character.
    let split = text[1..].find(['+', '-']).map(|offset| offset + 1)?;
    let (latitude_text, longitude_text) = text.split_at(split);

    Some((
        parse_sexagesimal(latitude_text, 2)?,
        parse_sexagesimal(longitude_text, 3)?,
    ))
}

/// `degree_digits` is 2 for latitude and 3 for longitude; the remaining digits
/// are minutes and optionally seconds.
fn parse_sexagesimal(text: &str, degree_digits: usize) -> Option<f64> {
    let sign = match text.as_bytes().first()? {
        b'+' => 1.0,
        b'-' => -1.0,
        _ => return None,
    };
    let digits = &text[1..];
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let degrees: f64 = digits.get(..degree_digits)?.parse().ok()?;
    let minutes: f64 = digits.get(degree_digits..degree_digits + 2)?.parse().ok()?;
    let seconds: f64 = match digits.get(degree_digits + 2..degree_digits + 4) {
        Some(text) if !text.is_empty() => text.parse().ok()?,
        _ => 0.0,
    };

    Some(sign * (degrees + minutes / 60.0 + seconds / 3600.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_parses_completely() {
        let parsed = places();
        // zone.tab carries a little over 400 zones; a large drop would mean the
        // parser is silently skipping lines.
        assert!(
            parsed.len() > 380,
            "only parsed {} places, expected the whole table",
            parsed.len()
        );

        let data_lines = ZONE_TAB
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .count();
        assert_eq!(
            parsed.len(),
            data_lines,
            "every non-comment line must yield a place"
        );
    }

    #[test]
    fn coordinates_are_within_range() {
        for place in places() {
            assert!(
                (-90.0..=90.0).contains(&place.latitude),
                "{} latitude {}",
                place.zone,
                place.latitude
            );
            assert!(
                (-180.0..=180.0).contains(&place.longitude),
                "{} longitude {}",
                place.zone,
                place.longitude
            );
        }
    }

    #[test]
    fn known_places_decode_correctly() {
        let kolkata = places().iter().find(|p| p.zone == "Asia/Kolkata").unwrap();
        // +2232+08822 is 22 deg 32' N, 88 deg 22' E.
        assert!((kolkata.latitude - 22.533_333).abs() < 1e-5);
        assert!((kolkata.longitude - 88.366_666).abs() < 1e-5);
        assert_eq!(kolkata.city, "Kolkata");
        assert_eq!(kolkata.country, "India");

        // A western, southern place, to check both signs.
        let santiago = places()
            .iter()
            .find(|p| p.zone == "America/Santiago")
            .unwrap();
        assert!(santiago.latitude < 0.0, "Santiago is south of the equator");
        assert!(santiago.longitude < 0.0, "Santiago is west of Greenwich");
    }

    #[test]
    fn multi_segment_zone_names_become_readable_cities() {
        assert_eq!(
            city_from_zone("America/Argentina/Rio_Gallegos"),
            "Rio Gallegos"
        );
        assert_eq!(city_from_zone("Asia/Kolkata"), "Kolkata");
        assert_eq!(city_from_zone("UTC"), "UTC");
    }

    #[test]
    fn both_coordinate_widths_parse() {
        // Minutes only.
        assert_eq!(
            parse_iso6709("+2232+08822"),
            Some((22.533_333_333_333_335, 88.36666666666666))
        );
        // With seconds.
        let (lat, lon) = parse_iso6709("+404251+0035523").unwrap();
        assert!((lat - 40.714_166_67).abs() < 1e-6);
        assert!((lon - 3.923_055_56).abs() < 1e-6);
    }

    #[test]
    fn malformed_coordinates_are_rejected_rather_than_guessed() {
        assert!(parse_iso6709("").is_none());
        assert!(parse_iso6709("2232+08822").is_none());
        assert!(parse_iso6709("+22ab+08822").is_none());
        assert!(parse_iso6709("+2232").is_none());
    }
}
