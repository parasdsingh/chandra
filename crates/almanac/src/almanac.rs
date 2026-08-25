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
use crate::day::DayOptions;
use crate::error::{Error, Result};
use crate::lunar::{self, LunarMonth, MonthSystem};
use crate::month::{self, DayDetail, GrahaMonth, IndexedMonth, MonthIndex, MonthLabel, MoonMonth};
use crate::phase::{self, PhaseName};
use crate::time::{self, CivilDay, DateKey};
use crate::tithi::CellTithi;
use crate::zodiac::{Nakshatra, Rashi};

/// Entries held before the coldest is dropped.
///
/// Twelve months covers a year of back-and-forth navigation plus the neighbours
/// prefetched around it, which is well past the point where a user is browsing
/// rather than scrubbing. Ten subjects, plus two entries a month that every
/// subject shares: the resolved grid and, in a lunar month, its tithis.
const CACHE_CAPACITY: usize = 12 * (1 + 9 + 2);

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
    /// Which weekday the grid opens on, zero-based from Monday, as the viewer's
    /// locale reports it. The grid is laid out here rather than in the front
    /// end, so the layout needs the one presentation fact this layer cannot
    /// derive.
    pub first_weekday: u8,
}

/// A month resolved to the 42 cells the grid draws, and what it is called.
#[derive(Debug, Clone)]
struct Resolved {
    grid: Vec<(CivilDay, bool)>,
    naming: MonthLabel,
}

impl Resolved {
    /// The month's first civil day, which identifies it uniquely in either
    /// system without needing a numbering scheme for lunar months.
    fn first_day(&self) -> DateKey {
        self.grid
            .iter()
            .find(|(_, in_month)| *in_month)
            .map(|(day, _)| day.date)
            .unwrap_or(self.grid[0].0.date)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CacheKey {
    // Keyed on the month's first civil day, which identifies a month uniquely in
    // either system without needing a numbering scheme for lunar months, plus
    // the weekday the grid opens on, which decides which 42 cells it holds.
    Moon(MonthSystem, DateKey, u8),
    Graha(Graha, MonthSystem, DateKey, u8),
    /// The lunar days of a grid, shared by every subject drawn on it.
    Frames(MonthSystem, DateKey, u8),
    /// A cursor resolved to a grid and a label.
    ///
    /// Keyed on the cursor and not on the month, because resolving is the step
    /// being avoided: it is what finds out which month the cursor names. A
    /// lunar cursor walks syzygies and builds 42 civil days to get there, and a
    /// warm month paid for all of it again on every open because the key was
    /// derived from its answer.
    Resolution(MonthSystem, i64, i32, u8),
}

#[derive(Debug, Clone)]
enum Cached {
    Moon(MoonMonth),
    Graha(Box<GrahaMonth>),
    Frames(Vec<CellTithi>),
    Resolution(Box<Resolved>),
}

/// The location and its zone, always read together.
///
/// Cloned once at the start of a request and used for the whole of it. Reading
/// it again part way through would let a location change land between the grid,
/// the tithis and the zone name, and produce a month assembled from two places.
#[derive(Clone)]
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

    /// Resolves a cursor, from the cache where possible.
    fn resolved(
        &self,
        cursor: MonthCursor,
        settings: &Settings,
        generation: u64,
    ) -> Result<Resolved> {
        let key = CacheKey::Resolution(
            cursor.system,
            cursor.anchor_unix_ms,
            cursor.offset,
            cursor.first_weekday,
        );
        if let Some(Cached::Resolution(cached)) = self.cached(key)? {
            return Ok(*cached);
        }

        let built = self.resolve(cursor, settings)?;
        self.store(key, Cached::Resolution(Box::new(built.clone())), generation)?;
        Ok(built)
    }

    /// Resolves a cursor to the 42 cells the grid draws and its display label.
    fn resolve(&self, cursor: MonthCursor, settings: &Settings) -> Result<Resolved> {
        let jd = chandra_ephemeris::unix_seconds_to_jd(cursor.anchor_unix_ms as f64 / 1000.0);

        if cursor.system == MonthSystem::Solar {
            let anchor_day = time::date_at(jd, &settings.zone)?;
            let (year, month_number) =
                shift_gregorian(anchor_day.year, anchor_day.month, cursor.offset);
            let days = month::month_days(year, month_number, &settings.zone)?;
            let name = gregorian_month_name(year, month_number)?;

            return Ok(Resolved {
                grid: time::grid_days(
                    days[0].date,
                    days[days.len() - 1].date,
                    cursor.first_weekday,
                    &settings.zone,
                )?,
                naming: MonthLabel {
                    label: format!("{name} {year}"),
                    name,
                    adhika: false,
                },
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
            naming: MonthLabel {
                // The era is named. Printed bare, "Shravana 2083" reads as a
                // year 57 in the future - especially in a panel whose solar
                // mode says "August 2026" in the same slot and whose cells are
                // annotated with Gregorian dates.
                label: format!("{} VS {}", month.display_name(), month.vikram_year),
                name: month.name.to_string(),
                adhika: month.adhika,
            },
            grid: time::grid_days(
                month.first_day,
                month.last_day,
                cursor.first_weekday,
                &settings.zone,
            )?,
        })
    }

    /// The lunar days of a grid, from the cache where possible.
    ///
    /// `None` in solar mode: a Gregorian calendar does not name tithis, and
    /// computing them to throw away would spend a sunrise on every one of 42
    /// cells for nothing.
    fn frames(
        &self,
        resolved: &Resolved,
        cursor: MonthCursor,
        settings: &Settings,
        generation: u64,
    ) -> Result<Option<Vec<CellTithi>>> {
        if !cursor.system.is_lunar() {
            return Ok(None);
        }

        let key = CacheKey::Frames(cursor.system, resolved.first_day(), cursor.first_weekday);
        if let Some(Cached::Frames(cached)) = self.cached(key)? {
            return Ok(Some(cached));
        }

        let built = month::tithi_frames(
            &self.engine,
            &resolved.grid,
            settings.location.observer,
            &settings.zone,
        )?;

        self.store(key, Cached::Frames(built.clone()), generation)?;
        Ok(Some(built))
    }

    /// The months of the year the cursor lands in, with the offset that reaches
    /// each one.
    ///
    /// The pointer route to a year. A wheel moves one month per notch, so
    /// without this the only way to 2140 is the keyboard, and the precision note
    /// the panel prints outside 1800-2399 is a promise that going there is
    /// possible.
    ///
    /// Offsets are against the cursor's own anchor, so the caller applies one by
    /// setting its offset to the value returned - no arithmetic on its side, and
    /// no assumption that a year holds twelve months. It does not, in lunar
    /// mode, twice a decade.
    pub fn month_index(&self, cursor: MonthCursor) -> Result<MonthIndex> {
        let settings = self.settings_snapshot()?;
        let jd = chandra_ephemeris::unix_seconds_to_jd(cursor.anchor_unix_ms as f64 / 1000.0);

        if cursor.system == MonthSystem::Solar {
            let anchor_day = time::date_at(jd, &settings.zone)?;
            let (year, month_number) =
                shift_gregorian(anchor_day.year, anchor_day.month, cursor.offset);

            // The offset that shows January of this year, and every month is a
            // step from there. A Gregorian year is twelve months by definition,
            // so this needs no search.
            let january = cursor.offset - (month_number as i32 - 1);
            let months = (1..=12)
                .map(|number| {
                    Ok(IndexedMonth {
                        name: gregorian_month_name(year, number)?,
                        offset: january + (number as i32 - 1),
                        adhika: false,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            return Ok(MonthIndex {
                year: year.to_string(),
                previous_year: january - 1,
                next_year: january + 12,
                months,
            });
        }

        let containing = lunar::month_containing(
            &self.engine,
            jd,
            cursor.system,
            settings.location.observer,
            &settings.zone,
        )?;
        let here = if cursor.offset == 0 {
            containing.clone()
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
        let year = here.vikram_year;

        // Walked rather than counted. A Vikram Samvat year holds twelve months
        // or thirteen, and which it is depends on whether a lunation fitted
        // inside one solar rashi - a fact only the ephemeris has. Walking out
        // from the month in hand until the year changes asks it directly, and
        // gets the adhika masa in its right position for free.
        //
        // The bound is a guard, not a count: a year cannot hold fifteen months,
        // so a walk that reaches fifteen is a bug and stops rather than spins.
        const GUARD: i32 = 15;
        let mut months: Vec<IndexedMonth> = Vec::with_capacity(13);
        let mut first_offset = cursor.offset;

        for step in 0..GUARD {
            let offset = cursor.offset - step;
            let month = self.lunar_at(&containing, offset, cursor, &settings)?;
            if month.vikram_year != year {
                break;
            }
            first_offset = offset;
            months.push(IndexedMonth {
                name: month.display_name(),
                offset,
                adhika: month.adhika,
            });
        }
        months.reverse();

        for step in 1..GUARD {
            let offset = cursor.offset + step;
            let month = self.lunar_at(&containing, offset, cursor, &settings)?;
            if month.vikram_year != year {
                break;
            }
            months.push(IndexedMonth {
                name: month.display_name(),
                offset,
                adhika: month.adhika,
            });
        }

        let last_offset = months
            .last()
            .map(|month| month.offset)
            .unwrap_or(cursor.offset);

        Ok(MonthIndex {
            year: format!("VS {year}"),
            previous_year: first_offset - 1,
            next_year: last_offset + 1,
            months,
        })
    }

    /// The lunar month `offset` steps from the cursor's anchor.
    fn lunar_at(
        &self,
        containing: &lunar::LunarMonth,
        offset: i32,
        cursor: MonthCursor,
        settings: &Settings,
    ) -> Result<lunar::LunarMonth> {
        if offset == 0 {
            return Ok(containing.clone());
        }
        lunar::shift(
            &self.engine,
            containing,
            offset,
            cursor.system,
            settings.location.observer,
            &settings.zone,
        )
    }

    pub fn moon_month(&self, cursor: MonthCursor) -> Result<MoonMonth> {
        // Both read before any computation starts. A reconfigure that lands
        // while this is running bumps the generation, and the result is then
        // discarded rather than stored under a configuration that did not
        // produce it.
        let settings = self.settings_snapshot()?;
        let generation = self.generation()?;

        let resolved = self.resolved(cursor, &settings, generation)?;
        let key = CacheKey::Moon(cursor.system, resolved.first_day(), cursor.first_weekday);
        if let Some(Cached::Moon(cached)) = self.cached(key)? {
            return Ok(cached);
        }

        let frames = self.frames(&resolved, cursor, &settings, generation)?;
        let built = month::moon_month(
            &self.engine,
            &resolved.grid,
            frames.as_deref(),
            resolved.naming,
        )?;

        self.store(key, Cached::Moon(built.clone()), generation)?;
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

        let settings = self.settings_snapshot()?;
        let generation = self.generation()?;

        let resolved = self.resolved(cursor, &settings, generation)?;
        let key = CacheKey::Graha(
            graha,
            cursor.system,
            resolved.first_day(),
            cursor.first_weekday,
        );
        if let Some(Cached::Graha(cached)) = self.cached(key)? {
            return Ok(*cached);
        }

        let frames = self.frames(&resolved, cursor, &settings, generation)?;
        let built = month::graha_month(
            &self.engine,
            graha,
            &resolved.grid,
            frames.as_deref(),
            resolved.naming,
        )?;

        self.store(key, Cached::Graha(Box::new(built.clone())), generation)?;
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

    /// Sankrantis between two instants.
    ///
    /// Nothing in the running app asks: the intercalary rule is decided inside
    /// `lunar::build`, from the same two Sun positions it needs anyway. This
    /// exists so `an_intercalary_month_contains_no_sankranti` can state the rule
    /// in its own terms rather than re-running the comparison the detection
    /// uses, which would assert only that the code agrees with itself.
    #[doc(hidden)]
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

    /// Detail for one day.
    ///
    /// The month system is a parameter because it decides whether the day has a
    /// panchanga at all: a Gregorian calendar names no tithi, so computing one
    /// would be work for a field the view would not show.
    /// A day, with whichever optional limbs the caller asked for.
    ///
    /// The month system is not a parameter any more. It used to decide whether
    /// the day carried a panchanga at all; it now carries one in both calendars,
    /// because yoga, karana and the muhurtas are facts about a civil day rather
    /// than about which calendar names the month it sits in.
    pub fn day_detail(
        &self,
        graha: Graha,
        date: DateKey,
        options: DayOptions,
    ) -> Result<DayDetail> {
        let settings = self.read_settings()?;
        let day = CivilDay::new(date, &settings.zone)?;
        month::day_detail(
            &self.engine,
            graha,
            &day,
            settings.location.observer,
            options,
        )
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

    fn settings_snapshot(&self) -> Result<Settings> {
        Ok(self.read_settings()?.clone())
    }

    fn generation(&self) -> Result<u64> {
        Ok(self.cache.lock().map_err(|_| poisoned())?.generation())
    }

    fn cached(&self, key: CacheKey) -> Result<Option<Cached>> {
        Ok(self.cache.lock().map_err(|_| poisoned())?.get(&key))
    }

    fn store(&self, key: CacheKey, value: Cached, generation: u64) -> Result<()> {
        self.cache
            .lock()
            .map_err(|_| poisoned())?
            .insert(key, value, generation);
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
///
/// Counted in `i64` and clamped, so an offset near the limits of `i32` cannot
/// overflow: it used to panic in a debug build and wrap to a nonsense year in a
/// release one. A clamped year is outside what the calendar can express, so
/// `DateKey::new` refuses it, which is the honest outcome for an impossible
/// request.
fn shift_gregorian(year: i16, month: i8, offset: i32) -> (i16, i8) {
    let zero_based = year as i64 * 12 + (month as i64 - 1) + offset as i64;
    (
        zero_based
            .div_euclid(12)
            .clamp(i16::MIN as i64, i16::MAX as i64) as i16,
        (zero_based.rem_euclid(12) + 1) as i8,
    )
}

/// `August`. The year is joined on by the caller, which is also the layer that
/// knows whether the year is Gregorian or Vikram Samvat.
fn gregorian_month_name(year: i16, month: i8) -> Result<String> {
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
    Ok((*name).to_string())
}

fn resolve_zone(name: &str) -> Result<TimeZone> {
    TimeZone::get(name).map_err(|_| Error::UnknownTimeZone(name.to_string()))
}

fn poisoned() -> Error {
    Error::Ephemeris(chandra_ephemeris::Error::Poisoned)
}
