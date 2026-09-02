use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// The tz database's zone table and country code table, embedded at build time
/// so the app never reads the host's copy. Depending on the host's `zoneinfo`
/// would make results vary with the machine's OS version.
const ZONE_TAB: &str = include_str!("../data/zone.tab");
const ISO3166_TAB: &str = include_str!("../data/iso3166.tab");

/// The GeoNames city list and its region names, trimmed by `tools/geonames.sh`
/// and embedded the same way.
///
/// 34,129 places against `zone.tab`'s 448, and coordinates to four decimal
/// places - about 11 metres - against its arcminutes, which are about 1.9 km.
/// D-007 promised this dataset from the beginning and it had never been built:
/// what shipped was one representative city per timezone, which is a list of
/// zones rather than a list of places (E5).
const CITIES_TSV: &str = include_str!("../data/cities.tsv");
const ADMIN1_TSV: &str = include_str!("../data/admin1.tsv");

/// A place the user can pick: an IANA zone with the coordinates of its
/// representative location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    /// IANA zone name, for example `Asia/Kolkata`.
    pub zone: String,
    pub city: String,
    /// The region the city is in, where the source names one.
    ///
    /// Not decoration: 1309 of the 34,129 city names in the list are not unique,
    /// so without it a search for Springfield offers the same word eight times
    /// with nothing to choose between them.
    pub region: Option<String>,
    pub country: String,
    pub country_code: String,
    /// Degrees north, negative south.
    pub latitude: f64,
    /// Degrees east, negative west.
    pub longitude: f64,
    /// Metres, from GeoNames' digital elevation model, where the source has one.
    ///
    /// `None` for a zone-table entry, which carries no height. Rise and set are
    /// computed at sea level without it, and the settings pane says so rather
    /// than reporting the assumption as a measurement (W-07).
    pub elevation: Option<f64>,
}

/// A city, with what is needed to find it but not to display it.
///
/// The lowercased name is held rather than made: `search` runs on every
/// keystroke, and lowercasing 34,129 names each time allocates 34,129 strings to
/// answer one letter.
pub(crate) struct City {
    pub(crate) place: Place,
    /// Ranks equal matches. A search for `york` should offer New York before
    /// York, Nebraska, and population is what says so.
    pub(crate) population: u64,
    /// The lowercased local name, and its lowercased ASCII form where the two
    /// differ. Both are searched: a reader who types `Cesky Tesin` means Cesky
    /// Tesin, and matching only `Ceský Tesín` finds nothing.
    pub(crate) folded: Vec<String>,
}

/// Every timezone's representative place, parsed once.
///
/// This is the zone table, not the city list. It is what answers "which place
/// stands for `Asia/Kolkata`", which is D-007's last step and the only thing the
/// app has before a location has been chosen. It is not what a search runs over.
pub fn zones() -> &'static [Place] {
    static ZONES: OnceLock<Vec<Place>> = OnceLock::new();
    ZONES.get_or_init(|| {
        let countries = parse_countries();
        let mut parsed: Vec<Place> = ZONE_TAB
            .lines()
            .filter_map(|line| parse_zone_line(line, &countries))
            .collect();
        parsed.sort_by(|a, b| a.city.cmp(&b.city).then_with(|| a.zone.cmp(&b.zone)));
        parsed
    })
}

/// Every city, parsed once.
pub(crate) fn cities() -> &'static [City] {
    static CITIES: OnceLock<Vec<City>> = OnceLock::new();
    CITIES.get_or_init(|| {
        let countries = parse_countries();
        let regions = parse_regions();
        CITIES_TSV
            .lines()
            .filter_map(|line| parse_city_line(line, &countries, &regions))
            .collect()
    })
}

/// Region code to region name, `IN.16` to `Maharashtra`.
fn parse_regions() -> Vec<(String, String)> {
    ADMIN1_TSV
        .lines()
        .filter_map(|line| {
            let (code, name) = line.split_once('\t')?;
            Some((code.to_string(), name.to_string()))
        })
        .collect()
}

/// One line of the trimmed GeoNames export.
///
/// The columns are named in `tools/geonames.sh`, which produced them.
fn parse_city_line(
    line: &str,
    countries: &[(String, String)],
    regions: &[(String, String)],
) -> Option<City> {
    let mut fields = line.split('\t');
    let city = fields.next()?;
    let ascii = fields.next()?;
    let country_code = fields.next()?;
    let region_code = fields.next()?;
    let zone = fields.next()?;
    let latitude: f64 = fields.next()?.parse().ok()?;
    let longitude: f64 = fields.next()?.parse().ok()?;
    let population: u64 = fields.next()?.parse().unwrap_or(0);
    let elevation = fields.next()?.parse::<f64>().ok();

    if city.is_empty() || zone.is_empty() {
        return None;
    }

    let key = format!("{country_code}.{region_code}");
    let region = regions
        .iter()
        .find(|(code, _)| *code == key)
        .map(|(_, name)| name.clone());

    let mut folded = vec![city.to_lowercase()];
    let plain = ascii.to_lowercase();
    if !plain.is_empty() && plain != folded[0] {
        folded.push(plain);
    }

    Some(City {
        folded,
        population,
        place: Place {
            zone: zone.to_string(),
            city: city.to_string(),
            region,
            country: countries
                .iter()
                .find(|(code, _)| code == country_code)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| country_code.to_string()),
            country_code: country_code.to_string(),
            latitude,
            longitude,
            elevation,
        },
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
        // The zone table names neither. A zone is not in a region, and it
        // carries no height - which is the whole of W-07: every city in the
        // world reported 0 m because this table has no column to report.
        region: None,
        elevation: None,
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
        let parsed = zones();
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
        for place in zones() {
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
        let kolkata = zones().iter().find(|p| p.zone == "Asia/Kolkata").unwrap();
        // +2232+08822 is 22 deg 32' N, 88 deg 22' E.
        assert!((kolkata.latitude - 22.533_333).abs() < 1e-5);
        assert!((kolkata.longitude - 88.366_666).abs() < 1e-5);
        assert_eq!(kolkata.city, "Kolkata");
        assert_eq!(kolkata.country, "India");

        // A western, southern place, to check both signs.
        let santiago = zones()
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
