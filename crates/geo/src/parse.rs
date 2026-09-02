use std::collections::HashMap;
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
#[derive(Clone)]
pub(crate) struct City {
    pub(crate) place: Place,
    /// Ranks matches that scored the same, largest first.
    ///
    /// Only among equals: an exact match on the name always beats a substring,
    /// so `york` offers York in England before New York City, which is correct -
    /// the user typed York. What population decides is which of the several
    /// Yorks in the United States comes first.
    pub(crate) population: u64,
    /// The lowercased local name, and its lowercased ASCII form where the two
    /// differ. Both are searched: a reader who types `Cesky Tesin` means Cesky
    /// Tesin, and matching only `Ceský Tesín` finds nothing.
    pub(crate) folded: Vec<String>,
    /// The region and country, lowercased.
    ///
    /// Held for the same reason `folded` is, and it was an oversight that they
    /// were not: `rank` lowercased them on every entry on every keystroke, which
    /// is tens of thousands of string allocations to answer one letter. The
    /// comment above `folded` gave the reason and the code beside it did the
    /// opposite.
    ///
    /// No folded zone. Nothing ranks on the zone any more: 34,129 cities share
    /// about 356 of them, so one zone match returned thousands of equal rows and
    /// `york` answered with Philadelphia.
    pub(crate) folded_region: Option<String>,
    pub(crate) folded_country: String,
}

/// Every timezone's representative place, parsed once.
///
/// This is the zone table, not the city list. It is what answers "which place
/// stands for `Asia/Kolkata`", which is D-007's last step and the only thing the
/// app has before a location has been chosen. It is not what a search runs over.
pub fn zones() -> &'static [Place] {
    static ZONES: OnceLock<Vec<Place>> = OnceLock::new();
    ZONES.get_or_init(|| {
        let mut parsed: Vec<Place> = ZONE_TAB.lines().filter_map(parse_zone_line).collect();
        parsed.sort_by(|a, b| a.city.cmp(&b.city).then_with(|| a.zone.cmp(&b.zone)));
        parsed
    })
}

/// Everything a search may offer: every city, plus the zone-table places no city
/// covers.
///
/// The city list has no entry for 62 of the 448 zones - Iqaluit, the Galapagos,
/// Kiritimati, Lord Howe, every Antarctic station - because none of them holds
/// 15,000 people. Searching cities alone silently made those zones unreachable
/// by hand, and there is no near-enough substitute: picking a city in a
/// neighbouring zone changes the timezone too.
///
/// The zone entry is only added where no city already answers to that name in
/// that country, so the precise row wins wherever there is one.
pub(crate) fn searchable() -> &'static [City] {
    static SEARCHABLE: OnceLock<Vec<City>> = OnceLock::new();
    SEARCHABLE.get_or_init(|| {
        let mut all: Vec<City> = cities().to_vec();

        let known: std::collections::HashSet<(String, String)> = all
            .iter()
            .map(|city| (city.folded[0].clone(), city.place.country_code.clone()))
            .collect();

        for place in zones() {
            let folded = place.city.to_lowercase();
            if known.contains(&(folded.clone(), place.country_code.clone())) {
                continue;
            }
            all.push(City {
                folded: vec![folded],
                // A zone's representative city has no population in the table and
                // needs none: it is here because nothing more precise exists, so
                // it should rank last among equal matches rather than first.
                population: 0,
                folded_region: None,
                folded_country: place.country.to_lowercase(),
                place: place.clone(),
            });
        }

        all
    })
}

/// Every city, parsed once.
pub(crate) fn cities() -> &'static [City] {
    static CITIES: OnceLock<Vec<City>> = OnceLock::new();
    CITIES.get_or_init(|| CITIES_TSV.lines().filter_map(parse_city_line).collect())
}

/// A name with its diacritics removed, or `None` if it has none.
///
/// Search folds case; it has to fold accents too. Without this, `Zurich`,
/// `Koln` and `Dusseldorf` find nothing at all - the list stores `Zürich`,
/// `Köln` and `Düsseldorf`, and a reader without an umlaut key types the first
/// spelling every time.
///
/// GeoNames' ASCII column does not cover it. That column transliterates the
/// German way, giving `Duesseldorf` and `Koeln`, which are the *other* spelling
/// somebody might type. Both are wanted, so both are kept: this produces the
/// stripped form and the ASCII column supplies the transliterated one.
///
/// Decomposition by table rather than by Unicode NFD, because the alternative is
/// a dependency for one column of one file. The table is the Latin-1 supplement
/// and Latin Extended-A, which is every accented letter the list actually uses
/// for a Latin-script name.
fn without_diacritics(name: &str) -> Option<String> {
    const MARKED: &str = "àáâãäåèéêëìíîïòóôõöùúûüýÿñçšžčćđłřůőűāēīōūăąęėįųğışțţďťľĺŕźżń";
    const PLAIN: [&str; 61] = [
        "a", "a", "a", "a", "a", "a", "e", "e", "e", "e", "i", "i", "i", "i", "o", "o", "o", "o",
        "o", "u", "u", "u", "u", "y", "y", "n", "c", "s", "z", "c", "c", "d", "l", "r", "u", "o",
        "u", "a", "e", "i", "o", "u", "a", "a", "e", "e", "i", "u", "g", "i", "s", "t", "t", "d",
        "t", "l", "l", "r", "z", "z", "n",
    ];

    let mut plain = String::with_capacity(name.len());
    let mut changed = false;
    for character in name.chars() {
        match MARKED.chars().position(|marked| marked == character) {
            Some(index) => {
                plain.push_str(PLAIN[index]);
                changed = true;
            }
            None => plain.push(character),
        }
    }

    changed.then_some(plain)
}

/// Whether a height is one the ephemeris will accept.
///
/// GeoNames writes **-9999** for "no value in the digital elevation model", and
/// 58 of the 34,129 rows carry it - Thessaloniki, Maracaibo and Bao'an among
/// them. Taken as a height it is not merely wrong, it is fatal: Swiss Ephemeris
/// refuses any observer outside -500 to 25000 metres, so `swe_rise_trans`
/// returns an error and *every* rise and set in the app fails, for as long as
/// that city is selected and across restarts. Picking Maracaibo broke the whole
/// calendar.
///
/// The bound is checked rather than the sentinel, because either would break the
/// same call and only one of them is documented by GeoNames. Nothing in the file
/// is legitimately outside it: the 138 genuinely negative rows - the Dead Sea,
/// the Caspian, the Netherlands - are all above -500.
fn usable_elevation(metres: &f64) -> bool {
    (-500.0..=25_000.0).contains(metres)
}

/// Region code to region name, `IN.16` to `Maharashtra`.
///
/// A map, not a list. Scanning a 3,865-entry list once per city is 132 million
/// comparisons, and it was measured at 850 milliseconds to two seconds - paid on
/// the first keystroke in the search field, inside a `OnceLock`, so it blocked.
/// The lookup is the whole of that cost: stubbed out, the same parse took 177 ms.
fn regions() -> &'static HashMap<String, String> {
    static REGIONS: OnceLock<HashMap<String, String>> = OnceLock::new();
    REGIONS.get_or_init(|| {
        ADMIN1_TSV
            .lines()
            .filter_map(|line| {
                let (code, name) = line.split_once('\t')?;
                Some((code.to_string(), name.to_string()))
            })
            .collect()
    })
}

/// One line of the trimmed GeoNames export.
///
/// The columns are named in `tools/geonames.sh`, which produced them.
fn parse_city_line(line: &str) -> Option<City> {
    let mut fields = line.split('\t');
    let city = fields.next()?;
    let ascii = fields.next()?;
    let country_code = fields.next()?;
    let region_code = fields.next()?;
    let zone = fields.next()?;
    let latitude: f64 = fields.next()?.parse().ok()?;
    let longitude: f64 = fields.next()?.parse().ok()?;
    let population: u64 = fields.next()?.parse().unwrap_or(0);
    let elevation = fields.next()?.parse::<f64>().ok().filter(usable_elevation);

    if city.is_empty() || zone.is_empty() {
        return None;
    }

    let region = regions()
        .get(&format!("{country_code}.{region_code}"))
        .cloned();

    // Three spellings at most, all lowercased: the local name, the same name with
    // its accents stripped, and GeoNames' own ASCII transliteration. They differ
    // more often than they look: `Köln` strips to `koln` and transliterates to
    // `koeln`, and somebody will type either.
    let mut folded = vec![city.to_lowercase()];
    if let Some(plain) = without_diacritics(&folded[0]) {
        folded.push(plain);
    }
    let transliterated = ascii.to_lowercase();
    if !transliterated.is_empty() && !folded.contains(&transliterated) {
        folded.push(transliterated);
    }

    let country = countries()
        .get(country_code)
        .cloned()
        .unwrap_or_else(|| country_code.to_string());

    Some(City {
        folded,
        population,
        folded_region: region.as_ref().map(|name| name.to_lowercase()),
        folded_country: country.to_lowercase(),
        place: Place {
            zone: zone.to_string(),
            city: city.to_string(),
            region,
            country,
            country_code: country_code.to_string(),
            latitude,
            longitude,
            elevation,
        },
    })
}

/// Country code to country name, parsed once.
///
/// A map rather than a list, and shared rather than built twice. Both tables
/// need it, and the city table asks 34,129 times.
fn countries() -> &'static HashMap<String, String> {
    static COUNTRIES: OnceLock<HashMap<String, String>> = OnceLock::new();
    COUNTRIES.get_or_init(|| {
        ISO3166_TAB
            .lines()
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| {
                let (code, name) = line.split_once('\t')?;
                Some((code.trim().to_string(), name.trim().to_string()))
            })
            .collect()
    })
}

fn parse_zone_line(line: &str) -> Option<Place> {
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
        country: countries()
            .get(country_code)
            .cloned()
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
    /// Every row of the city table parses.
    ///
    /// The zone table has had this guard since the beginning and the city table
    /// arrived without one. A row whose latitude fails to parse is dropped
    /// silently, and the file is regenerated by a script whose column order
    /// could change - so a reordering would quietly halve the list rather than
    /// fail.
    #[test]
    fn the_city_table_parses_completely() {
        let lines = super::CITIES_TSV.lines().filter(|l| !l.is_empty()).count();
        assert_eq!(
            super::cities().len(),
            lines,
            "every line of cities.tsv must yield a city"
        );
        assert!(lines > 30_000, "the table should hold the whole list");
    }

    /// No city carries a height the ephemeris would refuse.
    ///
    /// Checked against the old code: without the filter this finds 58 rows, and
    /// selecting any of them made every sunrise, sunset, moonrise and moonset in
    /// the app fail until the location was changed.
    #[test]
    fn no_city_carries_an_elevation_the_ephemeris_refuses() {
        let refused: Vec<&str> = super::cities()
            .iter()
            .filter(|city| {
                city.place
                    .elevation
                    .is_some_and(|metres| !(-500.0..=25_000.0).contains(&metres))
            })
            .map(|city| city.place.city.as_str())
            .collect();

        assert!(
            refused.is_empty(),
            "swe_rise_trans refuses these heights: {refused:?}"
        );
    }

    /// The sentinel is dropped rather than stored, and a real height is kept.
    #[test]
    fn an_unknown_elevation_is_none_and_a_real_one_survives() {
        let maracaibo = super::cities()
            .iter()
            .find(|city| city.place.city == "Maracaibo")
            .expect("Maracaibo is in the list");
        assert_eq!(
            maracaibo.place.elevation, None,
            "GeoNames writes -9999 here, which is not a height"
        );

        let bengaluru = super::cities()
            .iter()
            .find(|city| city.place.city == "Bengaluru")
            .expect("Bengaluru is in the list");
        assert!(bengaluru.place.elevation.is_some_and(|m| m > 800.0));
    }

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
