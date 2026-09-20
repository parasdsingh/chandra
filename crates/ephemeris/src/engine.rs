use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use swiss_eph as se;

use crate::body::{se_body, Graha};
use crate::error::{Error, Result};
use crate::observer::Observer;
use crate::sidereal::{NodeType, SiderealConfig};

// Swiss Ephemeris flag words. Defined here rather than taken from the binding
// crate because that crate types them inconsistently (some u32, some i32).
const SEFLG_SWIEPH: i32 = 2;
const SEFLG_MOSEPH: i32 = 4;
const SEFLG_SPEED: i32 = 256;
const SEFLG_SIDEREAL: i32 = 64 * 1024;

/// House system letter. `W` is whole sign: the ascendant's own rashi is the
/// first house and each house is one whole rashi, which is what jyotisha uses.
///
/// An earlier comment here claimed the ascendant does not depend on the system
/// because Swiss Ephemeris fills `ascmc[0]` before dividing the houses. That is
/// false, and an audit said so: `swehouse.c` rewrites `hsp->ac` on the whole
/// sign path, and the sidereal path substitutes `E` for `W` before dividing.
///
/// The value is right anyway. The rewrite is guarded by a sign test that a scan
/// of every ARMC from 0 to 360 at latitudes 51.5 to 90 never trips, so no
/// correction ever ships - but the safety is empirical, not structural, which is
/// a different claim from the one that was written here.
const SE_HSYS_WHOLE_SIGN: i32 = b'W' as i32;

/// Pseudo-body that makes `swe_calc_ut` return the obliquity and nutation
/// instead of a position.
const SE_ECL_NUT: i32 = -1;

const SE_CALC_RISE: i32 = 1;
const SE_CALC_SET: i32 = 2;

/// Standard atmosphere at sea level, used for the refraction model in rise and
/// set calculations.
///
/// **The observer's elevation has no effect on rise and set times as this is
/// called**, and the comment here used to say the opposite. Swiss Ephemeris
/// scales pressure for elevation only when `atpress` is passed as zero; passing
/// a literal disables that path, and `swe_rise_trans` applies no horizon dip of
/// its own. Measured at Bengaluru: sunrise at 920 m and at 8,848 m are both
/// 0.000 minutes from sunrise at sea level. With `atpress = 0.0` they separate
/// by 0.286 and 1.886 minutes.
///
/// Sea level is also the right horizon for an almanac (D-004), which is why
/// this has never been wrong in its output - only in its explanation. Changing
/// it would change every published rise and set time in the app and is a
/// decision, not a fix.
const ATMOSPHERIC_PRESSURE_MBAR: f64 = 1013.25;
const ATMOSPHERIC_TEMPERATURE_C: f64 = 15.0;

/// A date the bundled files must be able to serve: 2000-01-01, in the middle of
/// their 1800-2399 range.
const DATA_FILE_PROBE_JD: f64 = 2_451_545.0;

/// Size of the Swiss Ephemeris error buffer. The library's own headers require
/// at least 256 bytes and write a NUL-terminated string into it.
const ERROR_BUFFER_LEN: usize = 256;

/// Which theory actually produced a value.
///
/// Swiss Ephemeris falls back from its data files to the Moshier analytic theory
/// without raising an error, so this is derived from the flags it returns, not
/// from the flags requested. Moon positions differ between the two by about
/// 1 arcsecond, roughly 2 seconds of clock time at a nakshatra boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Full precision, served from the bundled `.se1` data files.
    Swieph,
    /// Reduced precision analytic fallback, used outside 1800-2399.
    Moshier,
}

impl Source {
    fn from_returned_flags(flags: i32) -> Self {
        if flags & SEFLG_MOSEPH != 0 {
            Source::Moshier
        } else {
            Source::Swieph
        }
    }

    /// The weaker of two provenances.
    pub fn weaker(self, other: Source) -> Source {
        match (self, other) {
            (Source::Swieph, Source::Swieph) => Source::Swieph,
            _ => Source::Moshier,
        }
    }

    /// Combines the provenance of several values into the weakest of them, so a
    /// composite result is never reported as more precise than its worst input.
    ///
    /// No values at all yields `Moshier`. Folding from `Swieph` instead made an
    /// empty iterator claim full precision from no evidence, which is the exact
    /// failure this type exists to prevent.
    pub fn weakest(values: impl IntoIterator<Item = Source>) -> Source {
        values
            .into_iter()
            .reduce(Source::weaker)
            .unwrap_or(Source::Moshier)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Sidereal ecliptic longitude in degrees, normalised to `[0, 360)`.
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Distance in astronomical units.
    pub distance: f64,
    /// Longitude change in degrees per day. Negative means retrograde.
    pub speed: f64,
    pub source: Source,
}

impl Position {
    pub fn is_retrograde(&self) -> bool {
        self.speed < 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Illumination {
    /// Illuminated fraction of the disc, `0.0` to `1.0`.
    pub fraction: f64,
    /// Sun-body-Earth angle in degrees. Zero at full, 180 at new.
    pub phase_angle: f64,
    /// Apparent angular distance from the Sun in degrees, `0` to `180`.
    pub elongation: f64,
    pub source: Source,
}

/// Rise and set for one civil day.
///
/// Both fields are optional and `None` is a legitimate result, not a failure:
/// the Moon fails to rise on roughly one day a month at any latitude because its
/// rising time drifts about 50 minutes later each day, and at high latitudes a
/// body can stay above or below the horizon for days.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiseSet {
    /// Julian Day (UT) of rising, if it occurs within the search window.
    pub rise: Option<f64>,
    /// Julian Day (UT) of setting, if it occurs within the search window.
    pub set: Option<f64>,
    /// Which theory served this body over this window (D-006).
    ///
    /// `None` for the nodes, which have no rise or set to attribute. A value
    /// here would be a statement about a reading that was never taken.
    ///
    /// `swe_rise_trans` reports a status code rather than a flag word, so it
    /// cannot answer for itself. This is read from a position call for the same
    /// body at the same instant - the quantity the rise search integrates - so
    /// it is observed rather than inferred from the date.
    pub source: Option<Source>,
}

/// A scalar reading, with the theory that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub degrees: f64,
    pub source: Source,
}

/// Guards the invariant that at most one [`Engine`] exists per process.
///
/// Swiss Ephemeris keeps its state in C globals, so a second engine would
/// silently reconfigure the first one's sidereal mode and its `Drop` would close
/// files still in use. Enforced rather than documented, because the failure is
/// invisible: results stay plausible while being computed under the wrong
/// ayanamsa.
static ENGINE_EXISTS: AtomicBool = AtomicBool::new(false);

struct Inner {
    config: SiderealConfig,
}

/// Serialised access to the Swiss Ephemeris C library.
///
/// The library keeps its ephemeris path, sidereal mode and observer position in
/// process-global variables and is not thread-safe. A single mutex guards every
/// entry point, and the global state is only ever mutated while that mutex is
/// held. Callers should invoke this from a blocking pool, never from a UI thread.
pub struct Engine {
    inner: Mutex<Inner>,
}

impl Engine {
    /// Points Swiss Ephemeris at the bundled data files and applies `config`.
    ///
    /// The path is required to exist. Swiss Ephemeris would otherwise accept a
    /// missing directory in silence and downgrade every result to Moshier, which
    /// is exactly the failure this crate exists to make impossible.
    pub fn new(ephemeris_path: &Path, config: SiderealConfig) -> Result<Self> {
        if ENGINE_EXISTS.swap(true, Ordering::SeqCst) {
            return Err(Error::AlreadyConstructed);
        }
        Self::construct(ephemeris_path, config).inspect_err(|_| {
            ENGINE_EXISTS.store(false, Ordering::SeqCst);
        })
    }

    fn construct(ephemeris_path: &Path, config: SiderealConfig) -> Result<Self> {
        if !ephemeris_path.is_dir() {
            return Err(Error::EphemerisPathMissing(ephemeris_path.to_path_buf()));
        }

        let path = ephemeris_path
            .to_str()
            .and_then(|s| CString::new(s).ok())
            .ok_or_else(|| Error::EphemerisPathInvalid(ephemeris_path.to_path_buf()))?;

        // SAFETY: `path` is a valid NUL-terminated C string that outlives the
        // call. Swiss Ephemeris copies the path into its own storage.
        unsafe { se::swe_set_ephe_path(path.as_ptr()) };

        let engine = Self {
            inner: Mutex::new(Inner { config }),
        };
        {
            let _guard = engine.inner.lock().map_err(|_| Error::Poisoned)?;
            set_sid_mode(config);

            // `is_dir` says a directory exists, not that anything is in it. An
            // empty one passes it, and Swiss Ephemeris then falls back to
            // Moshier for every date without raising anything - the silent
            // downgrade this whole crate exists to prevent, arriving through the
            // check meant to prevent it. So the files are asked for a date they
            // must be able to serve, and the flags they come back with decide.
            let (_, returned) = calc_raw(
                DATA_FILE_PROBE_JD,
                se_body::MOON,
                SEFLG_SWIEPH | SEFLG_SPEED,
                "ephemeris data probe",
            )?;
            if Source::from_returned_flags(returned) != Source::Swieph {
                return Err(Error::EphemerisDataUnusable(ephemeris_path.to_path_buf()));
            }
        }
        Ok(engine)
    }

    /// Changes the ayanamsa or node type.
    ///
    /// One critical section, as D-005 requires. The node type is read from
    /// `Inner` and the ayanamsa from a Swiss Ephemeris global; releasing the
    /// lock between the two writes lets a reader in on a pairing that was never
    /// selected, and the two differ by more than a degree.
    ///
    /// Callers must discard any cached results computed under the previous
    /// configuration; every sidereal longitude in the app depends on it.
    pub fn reconfigure(&self, config: SiderealConfig) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|_| Error::Poisoned)?;
        inner.config = config;
        set_sid_mode(config);
        Ok(())
    }

    pub fn config(&self) -> Result<SiderealConfig> {
        Ok(self.inner.lock().map_err(|_| Error::Poisoned)?.config)
    }

    /// Sidereal position of one graha at an instant.
    pub fn position(&self, jd_ut: f64, graha: Graha) -> Result<Position> {
        let inner = self.inner.lock().map_err(|_| Error::Poisoned)?;
        calc_position(jd_ut, graha, inner.config.node_type)
    }

    /// Sidereal positions of several grahas at one instant.
    ///
    /// Takes the lock once for the whole batch. Assembling a month of transit
    /// data calls this thousands of times, and per-body locking measurably
    /// dominates the cost at that volume.
    pub fn positions(&self, jd_ut: f64, grahas: &[Graha]) -> Result<Vec<Position>> {
        let inner = self.inner.lock().map_err(|_| Error::Poisoned)?;
        grahas
            .iter()
            .map(|&g| calc_position(jd_ut, g, inner.config.node_type))
            .collect()
    }

    /// Illuminated fraction and phase angle of the Moon.
    ///
    /// Phase is a geometric relationship between Sun, Moon and Earth, so it does
    /// not depend on the ayanamsa and is computed in the tropical frame.
    pub fn illumination(&self, jd_ut: f64) -> Result<Illumination> {
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;

        let mut attr = [0.0f64; 20];
        let mut err = [0 as c_char; ERROR_BUFFER_LEN];
        let flags = SEFLG_SWIEPH | SEFLG_SPEED;

        // SAFETY: called under the engine lock. `attr` has the 20 elements the
        // library documents it writes; `err` is a 256 byte buffer as required.
        let returned = unsafe {
            se::swe_pheno_ut(
                jd_ut,
                se_body::MOON,
                flags,
                attr.as_mut_ptr(),
                err.as_mut_ptr(),
            )
        };

        if returned < 0 {
            return Err(Error::Calculation {
                context: format!("moon illumination at JD {jd_ut}"),
                message: read_error(&err),
            });
        }

        Ok(Illumination {
            fraction: attr[1],
            phase_angle: attr[0],
            elongation: attr[2],
            source: Source::from_returned_flags(returned),
        })
    }

    /// Rise and set of a graha inside `[jd_ut_start, jd_ut_end)`.
    ///
    /// The window is given rather than assumed to be a day long. A civil day is
    /// 23 or 25 hours across a daylight saving change and only the caller knows
    /// which: taking it to be 1.0 discards a real event on the long day and
    /// reports one on both sides of the short one.
    ///
    /// Uses Swiss Ephemeris default geometry: the upper limb touching the
    /// horizon, with atmospheric refraction. That is the convention published
    /// rise and set tables use, so times here match an almanac
    /// (`docs/DECISIONS.md` D-004).
    pub fn rise_set(
        &self,
        jd_ut_start: f64,
        jd_ut_end: f64,
        graha: Graha,
        observer: Observer,
    ) -> Result<RiseSet> {
        if !observer.is_on_earth() {
            return Err(Error::InvalidObserver {
                latitude: observer.latitude,
                longitude: observer.longitude,
                elevation: observer.elevation,
            });
        }
        // The nodes are geometric points with no disc, and never cross the
        // horizon in the sense a rise and set calculation means. Answered before
        // the lock is taken and before anything is computed: there is no window
        // to search and so nothing whose provenance could be stated. Asking the
        // ephemeris first, only to throw the answer away, is work done to
        // produce a claim about a result that does not exist.
        if matches!(graha, Graha::Rahu | Graha::Ketu) {
            return Ok(RiseSet {
                rise: None,
                set: None,
                source: None,
            });
        }

        let inner = self.inner.lock().map_err(|_| Error::Poisoned)?;
        let body = se_body_id(graha, inner.config.node_type);

        let flags = SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL;

        let rise = calc_rise_event(jd_ut_start, body, SE_CALC_RISE, observer, graha)?;
        let set = calc_rise_event(jd_ut_start, body, SE_CALC_SET, observer, graha)?;

        // Provenance read at the events, not at the start of the search.
        //
        // The search runs forward, so an event can land the other side of the
        // data files' edge from where the window opened: a search begun on
        // 2400-01-10 reported Swieph for a rise on the 11th, which is Moshier.
        // That is precisely the claim D-006 exists to keep honest.
        //
        // Weakest of the instants that produced an answer, so a pair where one
        // event is analytic is not reported as though both were precise. The
        // window's own start is used when neither event exists, because the
        // provenance of "it does not rise" is still worth stating.
        let asked: Vec<f64> = [rise, set].into_iter().flatten().collect();
        let source = Some(Source::weakest(
            if asked.is_empty() {
                vec![jd_ut_start]
            } else {
                asked
            }
            .into_iter()
            .map(|at| -> Result<Source> {
                let (_, returned) = calc_raw(at, body, flags, graha.name())?;
                Ok(Source::from_returned_flags(returned))
            })
            .collect::<Result<Vec<_>>>()?,
        ));

        // Swiss Ephemeris searches forward without bound, so a body that does
        // not rise inside the window would otherwise report the next day's rise
        // as this one's.
        let within = |t: Option<f64>| t.filter(|&t| t < jd_ut_end);

        Ok(RiseSet {
            rise: within(rise),
            set: within(set),
            source,
        })
    }

    /// Sidereal longitude of the ascendant - the lagna - in degrees.
    ///
    /// The point of the ecliptic rising on the eastern horizon, which is what a
    /// Lagna Kundali is built around. It depends on the instant *and* on where
    /// the observer stands, unlike every other figure this engine returns: two
    /// people reading the same minute in different cities have different lagnas.
    ///
    /// Sidereal, on the configured ayanamsa. `swe_houses`, which the `swiss-eph`
    /// crate's own safe wrapper calls, takes no flags and so answers tropically:
    /// about 24 degrees out under Lahiri, and entirely plausible-looking. This
    /// calls `swe_houses_ex` directly for the same reason `calc_raw` calls
    /// `swe_calc_ut` directly, which is that the flag word is the whole point.
    pub fn ascendant(&self, jd_ut: f64, observer: Observer) -> Result<Reading> {
        if !observer.is_on_earth() {
            return Err(Error::InvalidObserver {
                latitude: observer.latitude,
                longitude: observer.longitude,
                elevation: observer.elevation,
            });
        }
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;
        let sidereal = houses_raw(HouseRequest {
            jd_ut,
            latitude: observer.latitude,
            longitude: observer.longitude,
            sidereal: true,
        })?;
        // The geometry is exact wherever the clock is, but the ayanamsa applied
        // to it is not: outside 1800-2399 `swe_houses_ex` resolves it from the
        // analytic fallback like everything else, and discards the flag word
        // that would have said so. Asked here instead, the same way `rise_set`
        // asks - one call whose answer is thrown away and whose flags are kept.
        //
        // `Source::Swieph` used to be hardcoded, which made the lagna the one
        // figure in the app that claimed exactness it had not checked.
        let flags = SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL;
        let (_, returned) = calc_raw(jd_ut, se_body::SUN, flags, "ascendant")?;

        Ok(Reading {
            degrees: sidereal.ascendant,
            source: Source::from_returned_flags(returned),
        })
    }

    /// The tropical ascendant, and the true obliquity, at an instant.
    ///
    /// Exposed for the test that checks [`Engine::ascendant`] against an
    /// independent derivation, and for the polar test. Both are facts about the
    /// sky rather than about a body, so neither is behind the sidereal
    /// configuration.
    pub fn ascendant_tropical(&self, jd_ut: f64, observer: Observer) -> Result<f64> {
        if !observer.is_on_earth() {
            return Err(Error::InvalidObserver {
                latitude: observer.latitude,
                longitude: observer.longitude,
                elevation: observer.elevation,
            });
        }
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;
        Ok(houses_raw(HouseRequest {
            jd_ut,
            latitude: observer.latitude,
            longitude: observer.longitude,
            sidereal: false,
        })?
        .ascendant)
    }

    /// True obliquity of the ecliptic in degrees, at an instant.
    pub fn obliquity(&self, jd_ut: f64) -> Result<f64> {
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;
        let (values, _) = calc_raw(jd_ut, SE_ECL_NUT, SEFLG_SWIEPH, "obliquity")?;
        Ok(values[0])
    }

    /// Greenwich apparent sidereal time in hours, at an instant.
    pub fn sidereal_time(&self, jd_ut: f64) -> Result<f64> {
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;
        // SAFETY: called under the engine lock. `swe_sidtime` takes a scalar and
        // returns one; it reads the same global state every other call does.
        Ok(unsafe { se::swe_sidtime(jd_ut) })
    }

    /// Ayanamsa in degrees at an instant, for display in settings.
    ///
    /// Signed, and it has to be: the Lahiri ayanamsa crosses zero around 285 CE,
    /// and `rem_euclid` turned a value a hair below it into 359.97 degrees - a
    /// number that reads as an enormous ayanamsa rather than a tiny negative
    /// one.
    ///
    /// Computed as the difference between the tropical and sidereal longitude of
    /// the Sun rather than read from `swe_get_ayanamsa_ex_ut`. Those two differ
    /// by up to the nutation in longitude (~17 arcseconds) because they resolve
    /// the equinox differently, and the difference is the quantity actually
    /// applied to every position this app displays. Showing a number that does
    /// not reconcile with the longitudes beside it would be a bug report waiting
    /// to happen.
    pub fn ayanamsa(&self, jd_ut: f64) -> Result<Reading> {
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;

        let tropical = calc_raw(jd_ut, se_body::SUN, SEFLG_SWIEPH | SEFLG_SPEED, "ayanamsa")?;
        let sidereal = calc_raw(
            jd_ut,
            se_body::SUN,
            SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL,
            "ayanamsa",
        )?;

        // Both calls are compared against what came back, as ARCHITECTURE says
        // every call is. Throwing the flag word away here made this the one
        // displayed figure that could be Moshier without saying so.
        Ok(Reading {
            // Folded to (-180, 180], not to [0, 360). The ayanamsa is a signed
            // quantity that passes through zero - around 285 CE for Lahiri -
            // and `rem_euclid` turned a value a hair below zero into 359.97
            // degrees, which reads as an enormous ayanamsa rather than a tiny
            // negative one.
            degrees: {
                let difference = (tropical.0[0] - sidereal.0[0]).rem_euclid(360.0);
                if difference > 180.0 {
                    difference - 360.0
                } else {
                    difference
                }
            },
            source: Source::from_returned_flags(tropical.1)
                .weaker(Source::from_returned_flags(sidereal.1)),
        })
    }

    /// Swiss Ephemeris version string, shown in the About pane so a support
    /// question can be answered without guessing which library shipped.
    pub fn library_version(&self) -> Result<String> {
        let _guard = self.inner.lock().map_err(|_| Error::Poisoned)?;
        let mut buf = [0 as c_char; ERROR_BUFFER_LEN];
        // SAFETY: called under the engine lock. The library writes a
        // NUL-terminated version string of at most 32 bytes into the buffer.
        unsafe { se::swe_version(buf.as_mut_ptr()) };
        Ok(read_error(&buf))
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Releases the library's cached file handles and internal buffers.
        // SAFETY: the engine is being dropped, so no other call can be in
        // flight through this handle, and no second engine can exist.
        unsafe { se::swe_close() };
        ENGINE_EXISTS.store(false, Ordering::SeqCst);
    }
}

/// Points Swiss Ephemeris' sidereal mode at `config`. Caller must hold the
/// engine lock.
fn set_sid_mode(config: SiderealConfig) {
    // SAFETY: called under the engine lock. `t0` and `ayan_t0` are ignored for
    // every predefined mode and only consulted for SE_SIDM_USER.
    unsafe { se::swe_set_sid_mode(config.ayanamsa.se_mode(), 0.0, 0.0) };
}

/// Maps a graha to the Swiss Ephemeris body used to compute it.
///
/// Ketu shares Rahu's body: it is the same node reflected by 180 degrees, which
/// [`calc_position`] applies afterwards.
fn se_body_id(graha: Graha, node_type: NodeType) -> i32 {
    match graha {
        Graha::Surya => se_body::SUN,
        Graha::Chandra => se_body::MOON,
        Graha::Mangala => se_body::MARS,
        Graha::Budha => se_body::MERCURY,
        Graha::Guru => se_body::JUPITER,
        Graha::Shukra => se_body::VENUS,
        Graha::Shani => se_body::SATURN,
        Graha::Rahu | Graha::Ketu => match node_type {
            NodeType::True => se_body::TRUE_NODE,
            NodeType::Mean => se_body::MEAN_NODE,
        },
    }
}

/// What a house calculation is asked for.
///
/// Named fields, not a positional pair of degrees. `swe_houses_ex` takes
/// **latitude before longitude**, which is the reverse of
/// [`Observer::as_se_geopos`] and of most of this codebase - and a transposed
/// call does not fail. It returns a real ascendant for a real place: Bengaluru
/// at 12.97N 77.59E transposes to 77.59N 12.97E, which is northern Norway, and
/// nothing at runtime looks wrong.
///
/// So the order is not something a reader has to remember. It is carried by the
/// field names, and `as_se_geopos` cannot be passed here because the types do
/// not line up.
struct HouseRequest {
    jd_ut: f64,
    latitude: f64,
    longitude: f64,
    /// Whether to apply the configured ayanamsa.
    sidereal: bool,
}

struct Houses {
    ascendant: f64,
}

/// The ascendant at an instant and a place.
///
/// `swe_houses_ex` rather than `swe_houses`: only the `_ex` form takes a flag
/// word, and without `SEFLG_SIDEREAL` the answer is tropical.
///
/// The sidereal branch depends on `swe_set_sid_mode` having been called, which
/// `Engine::construct` and `reconfigure` both do. If it had not been,
/// `swehouse.c` substitutes Fagan-Bradley silently rather than failing, so the
/// answer would be wrong by about 0.88 degrees and nothing here would return an
/// error. `crates/ephemeris/tests/ascendant.rs` is what holds that line: it
/// subtracts the sidereal lagna from the tropical one and asserts the remainder
/// is the ayanamsa the rest of the app applies, to the arcsecond. Asserted from
/// the answer rather than from the flag, because the flag is what would have
/// been ignored.
fn houses_raw(request: HouseRequest) -> Result<Houses> {
    let HouseRequest {
        jd_ut,
        latitude,
        longitude,
        sidereal,
    } = request;

    let mut cusps = [0.0f64; 13];
    let mut ascmc = [0.0f64; 10];
    let flags = if sidereal { SEFLG_SIDEREAL } else { 0 };

    // SAFETY: called under the engine lock. `cusps` has the 13 elements the
    // library writes for a 12 house system - it is 1-indexed and ignores [0] -
    // and `ascmc` the 10 it always writes. Latitude precedes longitude, which is
    // this function's whole reason for existing.
    let returned = unsafe {
        se::swe_houses_ex(
            jd_ut,
            flags,
            latitude,
            longitude,
            SE_HSYS_WHOLE_SIGN,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        )
    };

    if returned < 0 {
        return Err(Error::Calculation {
            context: format!("ascendant at JD {jd_ut}"),
            message: format!("house calculation failed at {latitude}, {longitude}"),
        });
    }

    Ok(Houses {
        ascendant: ascmc[0].rem_euclid(360.0),
    })
}

fn calc_raw(jd_ut: f64, body: i32, flags: i32, context: &str) -> Result<([f64; 6], i32)> {
    let mut xx = [0.0f64; 6];
    let mut err = [0 as c_char; ERROR_BUFFER_LEN];

    // SAFETY: called under the engine lock. `xx` has the 6 elements the library
    // writes when SEFLG_SPEED is set; `err` is a 256 byte buffer as required.
    let returned =
        unsafe { se::swe_calc_ut(jd_ut, body, flags, xx.as_mut_ptr(), err.as_mut_ptr()) };

    if returned < 0 {
        return Err(Error::Calculation {
            context: format!("{context} at JD {jd_ut}"),
            message: read_error(&err),
        });
    }
    Ok((xx, returned))
}

/// Caller must hold the engine lock.
fn calc_position(jd_ut: f64, graha: Graha, node_type: NodeType) -> Result<Position> {
    let body = se_body_id(graha, node_type);
    let flags = SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL;
    let (xx, returned) = calc_raw(jd_ut, body, flags, graha.name())?;

    // Ketu is always exactly opposite Rahu. Latitude and speed are shared: the
    // node pair moves as one, so only the longitude is reflected.
    let longitude = if graha == Graha::Ketu {
        (xx[0] + 180.0).rem_euclid(360.0)
    } else {
        xx[0].rem_euclid(360.0)
    };

    Ok(Position {
        longitude,
        latitude: xx[1],
        distance: xx[2],
        speed: xx[3],
        source: Source::from_returned_flags(returned),
    })
}

/// Caller must hold the engine lock.
///
/// Returns `Ok(None)` when the body does not cross the horizon, which Swiss
/// Ephemeris signals with `-2`. That is a real astronomical state, not an error.
fn calc_rise_event(
    jd_ut_start: f64,
    body: i32,
    event: i32,
    observer: Observer,
    graha: Graha,
) -> Result<Option<f64>> {
    let mut geopos = observer.as_se_geopos();
    let mut result = 0.0f64;
    let mut err = [0 as c_char; ERROR_BUFFER_LEN];

    // SAFETY: called under the engine lock. `geopos` is the 3 element array the
    // library requires, `result` a valid out-parameter, `err` a 256 byte buffer.
    // A null `starname` selects the planetary body given by `body`.
    let returned = unsafe {
        se::swe_rise_trans(
            jd_ut_start,
            body,
            std::ptr::null_mut(),
            SEFLG_SWIEPH,
            event,
            geopos.as_mut_ptr(),
            ATMOSPHERIC_PRESSURE_MBAR,
            ATMOSPHERIC_TEMPERATURE_C,
            &mut result,
            err.as_mut_ptr(),
        )
    };

    match returned {
        0 => Ok(Some(result)),
        -2 => Ok(None),
        _ => Err(Error::Calculation {
            context: format!("{} rise/set at JD {jd_ut_start}", graha.name()),
            message: read_error(&err),
        }),
    }
}

/// Reads Swiss Ephemeris' NUL-terminated error buffer.
fn read_error(buffer: &[c_char; ERROR_BUFFER_LEN]) -> String {
    // SAFETY: the library always writes a NUL-terminated string into the buffer,
    // and the buffer is zero-initialised, so a terminator is present either way.
    let text = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    text.to_string_lossy().trim().to_string()
}
