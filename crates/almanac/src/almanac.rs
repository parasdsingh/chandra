//! The facade the application layer talks to.
//!
//! Owns the single ephemeris engine, the current settings, and the month cache,
//! and is the only place those three are coordinated. Nothing above this needs
//! to know that changing an ayanamsa invalidates cached months, or that the
//! engine is process-global.

use std::path::Path;
use std::sync::{Mutex, RwLock};

use chandra_ephemeris::{Engine, Graha, Observer, SiderealConfig, Source};
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};

use crate::cache::Lru;
use crate::error::{Error, Result};
use crate::lunar::{self, LunarMonth, MonthSystem};
use crate::month::{self, DayDetail, GrahaMonth, MoonMonth};
use crate::phase::{self, PhaseName};
use crate::time::{CivilDay, DateKey};
use crate::zodiac::{Nakshatra, Rashi};

/// Months held per subject before the coldest is dropped.
///
/// Twelve covers a year of back-and-forth navigation plus the neighbours
/// prefetched around it, which is well past the point where a user is browsing
/// rather than scrubbing.
const CACHE_CAPACITY: usize = 12 * (1 + 9);

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub observer: Observer,
    /// IANA zone name, for example `Asia/Kolkata`.
    pub zone_name: String,
}

/// Where the calendar is pointing.
///
/// A lunar month has no year-and-number to index, so navigation is expressed as
/// "the month containing this instant, shifted by this many months". The same
/// cursor drives both systems, which is what lets the grid stay ignorant of
/// which one is in force.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonthCursor {
    pub anchor_unix_ms: i64,
    pub offset: i32,
    pub system: MonthSystem,
}

/// A month resolved to concrete days.
struct Resolved {
    days: Vec<CivilDay>,
    label: String,
    system: MonthSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CacheKey {
    // Keyed on the month's first civil day, which identifies a month uniquely in
    // either system without needing a numbering scheme for lunar months.
    Moon(MonthSystem, DateKey),
    Graha(Graha, MonthSystem, DateKey),
}

#[derive(Debug, Clone)]
enum Cached {
    Moon(MoonMonth),
    Graha(Box<GrahaMonth>),
}

struct Settings {
    location: Location,
    zone: TimeZone,
}

pub struct Almanac {
    engine: Engine,
    settings: RwLock<Settings>,
    cache: Mutex<Lru<CacheKey, Cached>>,
}

impl Almanac {
    pub fn new(
        ephemeris_path: &Path,
        location: Location,
        sidereal: SiderealConfig,
    ) -> Result<Self> {
        let zone = resolve_zone(&location.zone_name)?;
        Ok(Self {
            engine: Engine::new(ephemeris_path, sidereal)?,
            settings: RwLock::new(Settings { location, zone }),
            cache: Mutex::new(Lru::new(CACHE_CAPACITY)),
        })
    }

    /// Swiss Ephemeris version, for the About pane.
    pub fn library_version(&self) -> Result<String> {
        Ok(self.engine.library_version()?)
    }

    pub fn location(&self) -> Result<Location> {
        Ok(self.read_settings()?.location.clone())
    }

    pub fn sidereal(&self) -> Result<SiderealConfig> {
        Ok(self.engine.config()?)
    }

    /// Ayanamsa in degrees at an instant, for the Astrology pane.
    pub fn ayanamsa(&self, jd_ut: f64) -> Result<f64> {
        Ok(self.engine.ayanamsa(jd_ut)?)
    }

    pub fn set_location(&self, location: Location) -> Result<()> {
        let zone = resolve_zone(&location.zone_name)?;
        {
            let mut settings = self.settings.write().map_err(|_| poisoned())?;
            if settings.location == location {
                return Ok(());
            }
            *settings = Settings { location, zone };
        }
        self.invalidate()
    }

    pub fn set_sidereal(&self, sidereal: SiderealConfig) -> Result<()> {
        if self.engine.config()? == sidereal {
            return Ok(());
        }
        self.engine.reconfigure(sidereal)?;
        self.invalidate()
    }

    /// Resolves a cursor to the month's civil days and its display label.
    fn resolve(&self, cursor: MonthCursor) -> Result<Resolved> {
        let settings = self.read_settings()?;
        let jd = chandra_ephemeris::unix_seconds_to_jd(cursor.anchor_unix_ms as f64 / 1000.0);

        if cursor.system == MonthSystem::Solar {
            let anchor_day = CivilDay::new(DateKey::new(1, 1, 1)?, &settings.zone)?.date_of(jd)?;
            let shifted = shift_gregorian(anchor_day.year, anchor_day.month, cursor.offset);
            return Ok(Resolved {
                days: month::month_days(shifted.0, shifted.1, &settings.zone)?,
                label: gregorian_label(shifted.0, shifted.1)?,
                system: cursor.system,
            });
        }

        let containing = lunar::month_containing(
            &self.engine,
            jd,
            cursor.system,
            settings.location.observer,
            &settings.zone,
        )?;
        let month = if cursor.offset == 0 {
            containing
        } else {
            lunar::shift(
                &self.engine,
                &containing,
                cursor.offset,
                cursor.system,
                settings.location.observer,
                &settings.zone,
            )?
        };

        Ok(Resolved {
            label: format!("{} {}", month.display_name(), month.first_day.year),
            days: month::days_between(month.first_day, month.last_day, &settings.zone)?,
            system: cursor.system,
        })
    }

    pub fn moon_month(&self, cursor: MonthCursor) -> Result<MoonMonth> {
        let resolved = self.resolve(cursor)?;
        let key = CacheKey::Moon(resolved.system, resolved.days[0].date);
        if let Some(Cached::Moon(cached)) = self.cached(key)? {
            return Ok(cached);
        }

        let zone_name = self.read_settings()?.location.zone_name.clone();
        let built = month::moon_month(
            &self.engine,
            &resolved.days,
            &zone_name,
            resolved.label,
            resolved.system,
        )?;

        self.store(key, Cached::Moon(built.clone()))?;
        Ok(built)
    }

    pub fn graha_month(&self, graha: Graha, cursor: MonthCursor) -> Result<GrahaMonth> {
        if graha == Graha::Chandra {
            // The Moon has its own view; routing it here would compute transit
            // events nothing displays.
            return Err(Error::TimeZone(
                "the Moon is served by moon_month, not graha_month".into(),
            ));
        }

        let resolved = self.resolve(cursor)?;
        let key = CacheKey::Graha(graha, resolved.system, resolved.days[0].date);
        if let Some(Cached::Graha(cached)) = self.cached(key)? {
            return Ok(*cached);
        }

        let zone_name = self.read_settings()?.location.zone_name.clone();
        let built = month::graha_month(
            &self.engine,
            graha,
            &resolved.days,
            &zone_name,
            resolved.label,
            resolved.system,
        )?;

        self.store(key, Cached::Graha(Box::new(built.clone())))?;
        Ok(built)
    }

    /// The lunar month containing an instant.
    pub fn lunar_month_at(&self, jd: f64, system: MonthSystem) -> Result<LunarMonth> {
        let settings = self.read_settings()?;
        lunar::month_containing(
            &self.engine,
            jd,
            system,
            settings.location.observer,
            &settings.zone,
        )
    }

    /// The lunar month `offset` months from `month`.
    pub fn lunar_month_shift(
        &self,
        month: &LunarMonth,
        offset: i32,
        system: MonthSystem,
    ) -> Result<LunarMonth> {
        let settings = self.read_settings()?;
        lunar::shift(
            &self.engine,
            month,
            offset,
            system,
            settings.location.observer,
            &settings.zone,
        )
    }

    /// Sankrantis between two instants, for callers checking the intercalary
    /// rule directly.
    pub fn sankrantis_between(
        &self,
        from: f64,
        to: f64,
    ) -> Result<Vec<(f64, crate::zodiac::Rashi)>> {
        lunar::sankrantis(&self.engine, from, to)
    }

    /// The civil day after `date`, in the observer's zone.
    pub fn day_after(&self, date: DateKey) -> Result<DateKey> {
        let settings = self.read_settings()?;
        let day = CivilDay::new(date, &settings.zone)?;
        day.date_of(day.end_jd + 0.5)
    }

    pub fn day_detail(&self, graha: Graha, date: DateKey) -> Result<DayDetail> {
        let settings = self.read_settings()?;
        let day = CivilDay::new(date, &settings.zone)?;
        month::day_detail(&self.engine, graha, &day, settings.location.observer)
    }

    /// What the menu bar needs: the Moon's current phase, and where each enabled
    /// graha currently stands.
    pub fn now(&self, unix_ms: i64, subjects: &[Graha]) -> Result<Snapshot> {
        let jd = chandra_ephemeris::unix_seconds_to_jd(unix_ms as f64 / 1000.0);
        let illumination = self.engine.illumination(jd)?;
        let elongation = {
            let bodies = self.engine.positions(jd, &[Graha::Chandra, Graha::Surya])?;
            (bodies[0].longitude - bodies[1].longitude).rem_euclid(360.0)
        };

        let mut positions = Vec::with_capacity(subjects.len());
        for &graha in subjects {
            let position = self.engine.position(jd, graha)?;
            positions.push(SnapshotGraha {
                graha,
                longitude: position.longitude,
                rashi: Rashi::from_longitude(position.longitude),
                nakshatra: Nakshatra::from_longitude(position.longitude),
                retrograde: position.is_retrograde(),
                source: position.source,
            });
        }

        let source = Source::weakest(
            [illumination.source]
                .into_iter()
                .chain(positions.iter().map(|p| p.source)),
        );

        Ok(Snapshot {
            unix_ms,
            illumination: illumination.fraction,
            is_waxing: phase::is_waxing(elongation),
            phase: phase::intermediate_phase(elongation),
            grahas: positions,
            source,
        })
    }

    fn read_settings(&self) -> Result<std::sync::RwLockReadGuard<'_, Settings>> {
        self.settings.read().map_err(|_| poisoned())
    }

    fn cached(&self, key: CacheKey) -> Result<Option<Cached>> {
        Ok(self.cache.lock().map_err(|_| poisoned())?.get(&key))
    }

    fn store(&self, key: CacheKey, value: Cached) -> Result<()> {
        self.cache
            .lock()
            .map_err(|_| poisoned())?
            .insert(key, value);
        Ok(())
    }

    fn invalidate(&self) -> Result<()> {
        self.cache.lock().map_err(|_| poisoned())?.invalidate_all();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub unix_ms: i64,
    pub illumination: f64,
    pub is_waxing: bool,
    pub phase: PhaseName,
    pub grahas: Vec<SnapshotGraha>,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SnapshotGraha {
    pub graha: Graha,
    pub longitude: f64,
    pub rashi: Rashi,
    pub nakshatra: Nakshatra,
    pub retrograde: bool,
    pub source: Source,
}

/// Gregorian month arithmetic that carries across year boundaries.
fn shift_gregorian(year: i16, month: i8, offset: i32) -> (i16, i8) {
    let zero_based = year as i32 * 12 + (month as i32 - 1) + offset;
    (
        zero_based.div_euclid(12) as i16,
        (zero_based.rem_euclid(12) + 1) as i8,
    )
}

/// `August 2026`, from the calendar rather than a hardcoded name table.
fn gregorian_label(year: i16, month: i8) -> Result<String> {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let name = MONTHS.get(month as usize - 1).ok_or(Error::InvalidDate {
        year,
        month,
        day: 1,
    })?;
    Ok(format!("{name} {year}"))
}

fn resolve_zone(name: &str) -> Result<TimeZone> {
    TimeZone::get(name).map_err(|_| Error::UnknownTimeZone(name.to_string()))
}

fn poisoned() -> Error {
    Error::Ephemeris(chandra_ephemeris::Error::Poisoned)
}
