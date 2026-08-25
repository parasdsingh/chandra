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

const SE_CALC_RISE: i32 = 1;
const SE_CALC_SET: i32 = 2;

/// Standard atmosphere at sea level, used for the refraction model in rise and
/// set calculations. Swiss Ephemeris scales this for the observer's elevation.
const ATMOSPHERIC_PRESSURE_MBAR: f64 = 1013.25;
const ATMOSPHERIC_TEMPERATURE_C: f64 = 15.0;

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
    /// `swe_rise_trans` reports a status code rather than a flag word, so it
    /// cannot answer for itself. This is read from a position call for the same
    /// body at the same instant - the quantity the rise search integrates - so
    /// it is observed rather than inferred from the date.
    pub source: Source,
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
        let inner = self.inner.lock().map_err(|_| Error::Poisoned)?;
        let body = se_body_id(graha, inner.config.node_type);

        // One position call for the body, purely to read the flags Swiss
        // Ephemeris returns. It is asked for even when there can be no rise, so
        // the provenance of the window is always stated rather than assumed.
        let flags = SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL;
        let (_, returned) = calc_raw(jd_ut_start, body, flags, graha.name())?;
        let source = Source::from_returned_flags(returned);

        // The nodes are geometric points with no disc and never cross the
        // horizon in the sense a rise/set calculation means.
        if matches!(graha, Graha::Rahu | Graha::Ketu) {
            return Ok(RiseSet {
                rise: None,
                set: None,
                source,
            });
        }

        let rise = calc_rise_event(jd_ut_start, body, SE_CALC_RISE, observer, graha)?;
        let set = calc_rise_event(jd_ut_start, body, SE_CALC_SET, observer, graha)?;

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

    /// Ayanamsa in degrees at an instant, for display in settings.
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
            degrees: (tropical.0[0] - sidereal.0[0]).rem_euclid(360.0),
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

/// One `swe_calc_ut` call. Caller must hold the engine lock.
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
