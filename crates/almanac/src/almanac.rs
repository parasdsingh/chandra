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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CacheKey {
    Moon(i16, i8),
    Graha(Graha, i16, i8),
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

    pub fn moon_month(&self, year: i16, month: i8) -> Result<MoonMonth> {
        if let Some(Cached::Moon(cached)) = self.cached(CacheKey::Moon(year, month))? {
            return Ok(cached);
        }

        let settings = self.read_settings()?;
        let days = month::month_days(year, month, &settings.zone)?;
        let built = month::moon_month(&self.engine, &days, &settings.location.zone_name)?;
        drop(settings);

        self.store(CacheKey::Moon(year, month), Cached::Moon(built.clone()))?;
        Ok(built)
    }

    pub fn graha_month(&self, graha: Graha, year: i16, month: i8) -> Result<GrahaMonth> {
        if graha == Graha::Chandra {
            // The Moon has its own view; routing it here would compute transit
            // events nothing displays.
            return Err(Error::TimeZone(
                "the Moon is served by moon_month, not graha_month".into(),
            ));
        }
        if let Some(Cached::Graha(cached)) = self.cached(CacheKey::Graha(graha, year, month))? {
            return Ok(*cached);
        }

        let settings = self.read_settings()?;
        let days = month::month_days(year, month, &settings.zone)?;
        let built = month::graha_month(&self.engine, graha, &days, &settings.location.zone_name)?;
        drop(settings);

        self.store(
            CacheKey::Graha(graha, year, month),
            Cached::Graha(Box::new(built.clone())),
        )?;
        Ok(built)
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

fn resolve_zone(name: &str) -> Result<TimeZone> {
    TimeZone::get(name).map_err(|_| Error::UnknownTimeZone(name.to_string()))
}

fn poisoned() -> Error {
    Error::Ephemeris(chandra_ephemeris::Error::Poisoned)
}
