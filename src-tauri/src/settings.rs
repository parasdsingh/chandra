//! Persisted settings.
//!
//! Stored as plain JSON that this crate reads and writes itself, rather than
//! through a store plugin. Settings only ever cross the boundary through
//! commands, so a plugin would add a second, unguarded write path to the same
//! file for no benefit. Every field is explicit and every version change is an
//! explicit migration; there are no serde defaults quietly filling in gaps,
//! because a missing field usually means the schema moved and silently
//! substituting a default is how a user's location gets reset without warning.

use std::fs;
use std::path::{Path, PathBuf};

use chandra_almanac::day::DayOptions;
use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::varga::Varga;
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, SiderealConfig};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Bumped only when the shape changes in a way older files cannot satisfy.
pub const SCHEMA_VERSION: u32 = 13;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub location: LocationSetting,
    pub sidereal: SiderealSetting,
    pub calendar: CalendarSetting,
    pub panchanga: PanchangaSetting,
    pub chart: ChartSetting,
    pub tray: TraySetting,
    pub appearance: AppearanceSetting,
}

/// Which optional limbs the day view computes and shows.
///
/// Only the three that cost something are here. Dignity, drishti, planetary war
/// and the nakshatra lord have no switch: they cost one positions call between
/// them, and a switch for a field that is free is a decision asked of the user
/// for nothing.
///
/// All off by default. The day view is readable as it stands, and these are for
/// someone who came looking for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PanchangaSetting {
    pub yogas: bool,
    pub karanas: bool,
    pub muhurtas: bool,
}

impl From<PanchangaSetting> for DayOptions {
    fn from(setting: PanchangaSetting) -> Self {
        DayOptions {
            yogas: setting.yogas,
            karanas: setting.karanas,
            muhurtas: setting.muhurtas,
        }
    }
}

/// How large the whole panel is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSetting {
    /// Multiplier on every dimension: the window, the grid, the type, the marks.
    ///
    /// One number rather than a type-size setting, because a menu bar panel is a
    /// fixed composition - 42 cells of 40px in a 320px window - and growing the
    /// text inside a window that stayed put would only take space from the
    /// calendar. Clamped on the way in by [`AppearanceSetting::clamped`].
    pub scale: f64,
}

impl AppearanceSetting {
    /// Smallest and largest the panel may be drawn.
    ///
    /// Below 0.8 the 10px labels stop being legible; above 1.4 a 320 by 332
    /// panel grown to 448 by 465 starts to read as a window rather than as a
    /// menu bar popover, and on a laptop screen it crowds the menu bar it hangs
    /// from.
    pub const MINIMUM: f64 = 0.8;
    pub const MAXIMUM: f64 = 1.4;

    /// The scale, forced into range.
    ///
    /// A settings file is editable by hand and a bad value here would build a
    /// window of zero or of several thousand pixels, so the value is clamped
    /// where it is read rather than trusted where it was written.
    pub fn clamped(self) -> f64 {
        if self.scale.is_finite() {
            self.scale.clamp(Self::MINIMUM, Self::MAXIMUM)
        } else {
            1.0
        }
    }
}

/// The Lagna Kundali: which division it draws, and in which format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartSetting {
    /// Whether the compartments' contents slide as the lagna crosses its sign.
    ///
    /// On by default. The chart is a reading of now and did not look like one:
    /// redrawn every minute, every redraw identical to the last. Off leaves
    /// everything where the static layout puts it, and the caption still carries
    /// the degree, so nothing is lost but the motion.
    pub animate: bool,
    /// Which of the three chart formats is drawn. They differ in where a rashi
    /// is put on screen and not in what is true, so this changes the drawing and
    /// nothing else.
    ///
    /// There is no `tray` field: the chart is the one status item that is always
    /// there (D-030), so there is nothing to switch.
    pub format: ChartFormat,
    /// Divisions with their own menu bar item.
    ///
    /// Several at once, not one at a time: a practitioner reads D1 and D9
    /// together, and switching between them to compare is not reading them
    /// together. Each enabled division gets its own status item and its own
    /// panel, exactly as each enabled graha does.
    ///
    /// D1 is always in here. It is the chart the app has always had and the one
    /// D-030 made permanent; the other fifteen are switches.
    pub vargas: Vec<Varga>,
    /// Whether a North Indian compartment carries the sign's number rather than
    /// its name.
    ///
    /// Both Drik Panchang and Jagannatha Hora write a number. A number is a
    /// lookup, though, and this app has spent a lot of effort on not making a
    /// reader do one - so the name is the default and the number is the choice.
    /// Drik Panchang offers the same switch the other way round.
    pub numbered: bool,
    /// Whether the North Indian chart draws the degree lines a compartment is
    /// laid out on.
    ///
    /// Off by default, and it is the only setting in the app that turns
    /// something *on* rather than choosing between two readings: it draws a
    /// construction line, and a chart is not improved for most readers by
    /// showing its own scaffolding. It is here because the construction is a
    /// real answer to a real question - why two bodies in the same sign are
    /// drawn where they are, and why a crowded house has to crowd - and that
    /// question is asked by exactly the reader who would go looking in
    /// Advanced.
    ///
    /// `docs/design/traversal.md`. Ignored by the two grid formats, whose
    /// compartments are the signs and have no route through them.
    pub grid: bool,
}

/// The three chart formats in common use.
///
/// They differ in where on screen a rashi is drawn and what is written in the
/// compartment. They do not differ in what is true, which is why one payload
/// serves all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartFormat {
    /// Houses fixed, signs move. House 1 is the top diamond. Cannot be drawn
    /// without a lagna, which is why a location is mandatory.
    #[default]
    North,
    /// Signs fixed, houses move. Meena top-left, clockwise.
    South,
    /// Same geometry as South, different origin and direction.
    East,
}

impl ChartFormat {
    pub const ALL: [ChartFormat; 3] = [ChartFormat::North, ChartFormat::South, ChartFormat::East];

    pub const fn key(self) -> &'static str {
        match self {
            ChartFormat::North => "north",
            ChartFormat::South => "south",
            ChartFormat::East => "east",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ChartFormat::North => "North Indian",
            ChartFormat::South => "South Indian",
            ChartFormat::East => "East Indian",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarSetting {
    /// Whether months run Gregorian, new moon to new moon, or full moon to full
    /// moon.
    pub month_system: MonthSystem,
    /// Which ingress, if either, is labelled on the grid.
    pub ingress: IngressSetting,
}

/// Which division a cell names when the subject enters one.
///
/// One or the other, never both: the label takes the glyph's place in a 40px
/// cell, and there is one glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngressSetting {
    #[default]
    Off,
    Rashi,
    Nakshatra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationSetting {
    pub mode: LocationMode,
    /// Present when `mode` is `Manual`, and also kept as the last known
    /// automatic result so a failed resolution does not lose the previous one.
    pub place: Option<PlaceSetting>,
    /// Metres above sea level, applied on top of whichever step of the chain
    /// resolved the location.
    ///
    /// Its own field rather than part of `place`, because it is the one observer
    /// property no step of the chain supplies: `zone.tab` carries no elevation at
    /// all and CoreLocation's vertical fix is poor. Editing it used to mean
    /// writing a whole fabricated place, which then reported itself as having
    /// come from the device.
    pub elevation: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationMode {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceSetting {
    pub label: String,
    pub zone: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Metres above sea level, where the step that resolved this place knew.
    ///
    /// `None` is not zero. The city table carries no elevation column, so a
    /// place picked from search has none; CoreLocation does supply one. As an
    /// `f64` the two were the same value, and a city at sea level and a city
    /// whose height nobody knows both printed `0 m` - one a measurement, the
    /// other a guess wearing a measurement's clothes.
    ///
    /// Affects rise and set by roughly four minutes at 900 m, so it is a real
    /// input rather than a decoration.
    pub elevation: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiderealSetting {
    pub ayanamsa: Ayanamsa,
    pub node_type: NodeType,
}

impl From<SiderealSetting> for SiderealConfig {
    fn from(setting: SiderealSetting) -> Self {
        SiderealConfig {
            ayanamsa: setting.ayanamsa,
            node_type: setting.node_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraySetting {
    /// Calendars with their own menu bar item.
    ///
    /// Chandra is in here like any other since D-030. It used to be excluded and
    /// permanent, which is why a file written before schema 9 never names it and
    /// the migration has to put it back.
    pub subjects: Vec<Graha>,
    /// Draw tray icons in a fixed colour instead of as template images. Off by
    /// default: a fixed colour is invisible in one of the two menu bar
    /// appearances (D-008).
    pub colour_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            location: LocationSetting {
                mode: LocationMode::Automatic,
                place: None,
                elevation: None,
            },
            sidereal: SiderealSetting {
                ayanamsa: Ayanamsa::Lahiri,
                // The mean node, because that is what a panchanga uses. The
                // siddhantic model defines Rahu as a uniformly retrograde point
                // - "always vakri" is the definition rather than an observation
                // - and the mean node is that definition computed. The
                // Rashtriya Panchang, Lahiri's ephemeris and KP all publish it.
                //
                // The true node is the real osculating intersection of the
                // Moon's orbital plane with the ecliptic, which is where
                // eclipses happen. It is also a perturbation the classical
                // model does not contain: it oscillates about the mean by up to
                // 1.6 degrees and turns direct about twenty-five times a year
                // (D-026). Shipping it as the default made Chandra disagree
                // with any panchanga laid beside it.
                //
                // Only new installs are affected. A settings file that already
                // names a node type keeps it: this is the value nobody chose,
                // not a correction to one somebody did.
                node_type: NodeType::Mean,
            },
            calendar: CalendarSetting {
                month_system: MonthSystem::Solar,
                ingress: IngressSetting::Off,
            },
            panchanga: PanchangaSetting::default(),
            chart: ChartSetting {
                format: ChartFormat::North,
                numbered: false,
                animate: true,
                grid: false,
                vargas: vec![Varga::D1],
            },
            tray: TraySetting {
                // The moon, which is now listed here like any other calendar and
                // is the only one on by default. The chart is not in this list:
                // it is always there and has no switch.
                subjects: vec![Graha::Chandra],
                colour_mode: false,
            },
            appearance: AppearanceSetting { scale: 1.0 },
        }
    }
}

impl Settings {
    /// A value with every optional field populated.
    ///
    /// Used by the IPC contract test so the recorded shape shows what `place`
    /// contains rather than a bare null.
    #[doc(hidden)]
    pub fn sample() -> Self {
        Self {
            location: LocationSetting {
                mode: LocationMode::Manual,
                place: Some(PlaceSetting {
                    label: "Bengaluru".into(),
                    zone: "Asia/Kolkata".into(),
                    latitude: 12.9716,
                    longitude: 77.5946,
                    elevation: Some(920.0),
                }),
                elevation: Some(940.0),
            },
            calendar: CalendarSetting {
                month_system: MonthSystem::Amanta,
                ingress: IngressSetting::Rashi,
            },
            tray: TraySetting {
                subjects: vec![Graha::Mangala],
                colour_mode: false,
            },
            ..Self::default()
        }
    }

    pub fn path(config_dir: &Path) -> PathBuf {
        config_dir.join("settings.json")
    }

    /// Reads settings, falling back to defaults when there is nothing to read.
    ///
    /// A file that exists but cannot be parsed is an error rather than a silent
    /// reset: overwriting a user's configuration because one field went bad
    /// loses information they cannot get back.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = Self::path(config_dir);
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| AppError::Settings(format!("cannot read {}: {e}", path.display())))?;
        let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            AppError::Settings(format!("{} is not valid JSON: {e}", path.display()))
        })?;

        let version = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AppError::Settings("settings file has no schema_version".into()))?;

        let migrated = migrate(value, version as u32)?;
        serde_json::from_value(migrated)
            .map_err(|e| AppError::Settings(format!("settings do not match the schema: {e}")))
    }

    pub fn save(&self, config_dir: &Path) -> Result<()> {
        fs::create_dir_all(config_dir).map_err(|e| {
            AppError::Settings(format!("cannot create {}: {e}", config_dir.display()))
        })?;

        let path = Self::path(config_dir);
        let body = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Settings(format!("cannot serialise settings: {e}")))?;

        // Write to a sibling then rename, so an interrupted write cannot leave a
        // truncated settings file behind.
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, body).map_err(|e| {
            AppError::Settings(format!("cannot write {}: {e}", temporary.display()))
        })?;
        fs::rename(&temporary, &path)
            .map_err(|e| AppError::Settings(format!("cannot replace {}: {e}", path.display())))?;

        Ok(())
    }
}

/// Brings a settings document up to [`SCHEMA_VERSION`].
///
/// Each step is written explicitly. There is deliberately no "unknown version,
/// use defaults" branch: that path is how a downgrade silently erases settings a
/// newer build wrote.
fn migrate(mut value: serde_json::Value, from: u32) -> Result<serde_json::Value> {
    let mut version = from;

    // 1 -> 2. Elevation became the observer's own correction rather than part of
    // the place, so it can be set without inventing a location that then
    // reported itself as having come from the device.
    //
    // A version 1 place carried the elevation every rise and set was computed
    // with, so it moves across rather than being left where it sat. Leaving it
    // behind looked harmless - the place still resolved with it - right up until
    // the user picked a different city, at which point the new place carried no
    // elevation, the correction was still absent, and their metres vanished
    // without a word. Carrying it over costs a device-supplied elevation being
    // relabelled as the user's own, which they can clear; the alternative loses
    // data silently.
    //
    // `launch_at_login` and `time_format` were removed in the same step and need
    // no clause: neither was ever settable, and serde ignores a field that is no
    // longer declared.
    if version == 1 {
        let location = value
            .get_mut("location")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| AppError::Settings("settings schema 1 has no location block".into()))?;

        let carried = location
            .get("place")
            .and_then(|place| place.get("elevation"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        location.insert("elevation".into(), carried);
        value["schema_version"] = serde_json::Value::from(2u32);
        version = 2;
    }

    // 2 -> 3. The panel gained a size. A file written before it means the size
    // it was drawn at, which is 1.0.
    if version == 2 {
        value
            .as_object_mut()
            .ok_or_else(|| AppError::Settings("settings are not an object".into()))?
            .insert("appearance".into(), serde_json::json!({ "scale": 1.0 }));
        value["schema_version"] = serde_json::Value::from(3u32);
        version = 3;
    }

    // 3 -> 4. The day view gained yoga, karana and the muhurtas, each behind a
    // switch. A file written before them means the state they were in, which is
    // off: none of the three existed to be switched on.
    //
    // Inserted rather than left to serde's default, so a document that has been
    // through the migration is complete on disk. A missing block that only ever
    // resolved to a default would be indistinguishable from one a future step
    // needs to read, and this is the file those steps are written against.
    if version == 3 {
        value
            .as_object_mut()
            .ok_or_else(|| AppError::Settings("settings are not an object".into()))?
            .insert(
                "panchanga".into(),
                serde_json::json!({ "yogas": false, "karanas": false, "muhurtas": false }),
            );
        value["schema_version"] = serde_json::Value::from(4u32);
        version = 4;
    }

    // 4 -> 5. The grid gained ingress labels. A file written before them means
    // the state they were in, which is off: D-024 had taken the ingress markers
    // off the grid entirely, and there was nothing to label.
    if version == 4 {
        value
            .get_mut("calendar")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| AppError::Settings("settings schema 4 has no calendar block".into()))?
            .insert("ingress".into(), serde_json::Value::from("off"));
        value["schema_version"] = serde_json::Value::from(5u32);
        version = 5;
    }

    // 5 -> 6. A place's elevation became optional, because "not known" and
    // "measured as zero" were the same `f64` and both printed `0 m`.
    //
    // A version 5 place has a number, and it is kept: whatever wrote it did so
    // deliberately, and it is the height every rise and set in the app has been
    // computed with. A place is not rewritten as "unknown" here on the grounds
    // that it *might* have been a default - that would discard a real
    // measurement to correct a presentation, and the user would watch their
    // metres disappear a second time (the same mistake the 1 -> 2 step exists
    // to have avoided).
    if version == 5 {
        if let Some(place) = value
            .get_mut("location")
            .and_then(|location| location.get_mut("place"))
            .and_then(serde_json::Value::as_object_mut)
        {
            // Already `Option`-shaped in JSON: a number stays a number and
            // deserialises as `Some`. Only an absent key needs writing, so a
            // place written without one becomes an explicit null rather than
            // relying on serde's default.
            place.entry("elevation").or_insert(serde_json::Value::Null);
        }
        value["schema_version"] = serde_json::Value::from(6u32);
        version = 6;
    }

    // 6 -> 7. The Lagna Kundali arrived, with its own menu bar item and a choice
    // of three formats. A file written before it means the state it was in,
    // which is that the feature did not exist - but the item ships on, so an
    // existing install gets it too rather than having to find a switch for
    // something it has never seen.
    if version == 6 {
        value
            .as_object_mut()
            .ok_or_else(|| AppError::Settings("settings are not an object".into()))?
            .insert(
                "chart".into(),
                serde_json::json!({ "tray": true, "format": "north", "numbered": false }),
            );
        value["schema_version"] = serde_json::Value::from(7u32);
        version = 7;
    }

    // 7 -> 8. The chart gained a choice of caption - the sign's name or its
    // number - and every field on this struct is required.
    //
    // The field was first added *without* a version bump, on the reasoning that
    // 6 -> 7 already wrote the chart block. It does, but only for a file that
    // was at 6: anything already migrated to 7 by an earlier build had a chart
    // block with two fields and no third, so it stopped deserialising and the
    // app refused to start. A new required field always needs a new version,
    // whatever the last one happened to write.
    if version == 7 {
        if let Some(chart) = value
            .get_mut("chart")
            .and_then(serde_json::Value::as_object_mut)
        {
            chart
                .entry("numbered")
                .or_insert(serde_json::Value::Bool(false));
        }
        value["schema_version"] = serde_json::Value::from(8u32);
        version = 8;
    }

    // 8 -> 9. The chart became the permanent status item and every calendar
    // became toggleable, the moon included (D-030).
    //
    // Two things move. The moon was permanent and so was never written to
    // `subjects`; every existing install has been showing it, so it goes in at
    // the front rather than silently switching off on upgrade. And `chart.tray`
    // is gone - serde ignores a field that is no longer declared, so it needs no
    // clause here, but its meaning does: an install that had deliberately turned
    // the chart off gets it back, because there is no longer a switch for it.
    // That is the point of the change rather than a side effect of it.
    if version == 8 {
        let subjects = value
            .get_mut("tray")
            .and_then(serde_json::Value::as_object_mut)
            .and_then(|tray| tray.get_mut("subjects"))
            .and_then(serde_json::Value::as_array_mut)
            .ok_or_else(|| {
                // Named for the field rather than for a version: this step runs
                // for a document that entered the chain at any version from 1,
                // and saying "schema 8" of a schema 1 file names the wrong one.
                AppError::Settings("settings have no tray subjects to migrate".into())
            })?;

        let moon = serde_json::Value::from(Graha::Chandra.key());
        if !subjects.contains(&moon) {
            subjects.insert(0, moon);
        }
        value["schema_version"] = serde_json::Value::from(9u32);
        version = 9;
    }

    // 9 -> 10. The chart gained a division. A file written before it means the
    // chart it was drawing, which is the rashi chart.
    if version == 9 {
        if let Some(chart) = value
            .get_mut("chart")
            .and_then(serde_json::Value::as_object_mut)
        {
            chart
                .entry("varga")
                .or_insert(serde_json::Value::from("d1"));
        }
        value["schema_version"] = serde_json::Value::from(10u32);
        version = 10;
    }

    // 10 -> 11. One division became a set of them, because several charts can be
    // in the menu bar at once. The division the file names is carried over as the
    // set's only member, and D1 is added if it was something else - the rashi
    // chart is the permanent item and there has to be one.
    if version == 10 {
        if let Some(chart) = value
            .get_mut("chart")
            .and_then(serde_json::Value::as_object_mut)
        {
            let chosen = chart
                .get("varga")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("d1")
                .to_string();

            let mut vargas = vec![serde_json::Value::from("d1")];
            if chosen != "d1" {
                vargas.push(serde_json::Value::from(chosen));
            }
            chart.remove("varga");
            chart.insert("vargas".into(), serde_json::Value::Array(vargas));
        }
        value["schema_version"] = serde_json::Value::from(11u32);
        version = 11;
    }

    // 11 -> 12. The chart gained motion. A file written before it was drawn
    // still, but the feature ships on: a reader who wants it off can say so, and
    // one who has never seen it cannot ask for it.
    if version == 11 {
        if let Some(chart) = value
            .get_mut("chart")
            .and_then(serde_json::Value::as_object_mut)
        {
            chart
                .entry("animate")
                .or_insert(serde_json::Value::Bool(true));
        }
        value["schema_version"] = serde_json::Value::from(12u32);
        version = 12;
    }

    // 12 -> 13. The chart gained the degree grid, off by default. A file
    // written before it existed keeps the chart it already had: this one is not
    // like `animate`, which shipped on because a reader who has never seen a
    // thing cannot ask for it. The grid is scaffolding, and scaffolding that
    // appeared unasked over an existing reader's chart would be a change to
    // what their chart looks like, not an addition to what it can do.
    if version == 12 {
        if let Some(chart) = value
            .get_mut("chart")
            .and_then(serde_json::Value::as_object_mut)
        {
            chart
                .entry("grid")
                .or_insert(serde_json::Value::Bool(false));
        }
        value["schema_version"] = serde_json::Value::from(13u32);
        version = 13;
    }

    match version {
        SCHEMA_VERSION => Ok(value),
        newer if newer > SCHEMA_VERSION => Err(AppError::Settings(format!(
            "settings were written by a newer version of Chandra (schema {newer}, this build \
             understands {SCHEMA_VERSION})"
        ))),
        older => Err(AppError::Settings(format!(
            "no migration exists from settings schema {older}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chandra-settings-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn absent_settings_yield_defaults() {
        let dir = temp_dir("absent");
        let settings = Settings::load(&dir).expect("defaults");
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.sidereal.ayanamsa, Ayanamsa::Lahiri);
        // The mean node, which is what a panchanga uses. Why is in
        // `a_new_install_uses_the_node_a_panchanga_uses`.
        assert_eq!(settings.sidereal.node_type, NodeType::Mean);
        // The moon alone, and it is in the list rather than beside it: every
        // calendar is toggleable since D-030, and this is the one switched on.
        assert_eq!(settings.tray.subjects, vec![Graha::Chandra]);
        assert!(!settings.tray.colour_mode);
    }

    #[test]
    fn settings_round_trip() {
        let dir = temp_dir("roundtrip");
        let mut settings = Settings::default();
        settings.tray.subjects = vec![Graha::Mangala, Graha::Shani];
        settings.location = LocationSetting {
            mode: LocationMode::Manual,
            place: Some(PlaceSetting {
                label: "Bengaluru".into(),
                zone: "Asia/Kolkata".into(),
                latitude: 12.9716,
                longitude: 77.5946,
                elevation: Some(920.0),
            }),
            elevation: Some(940.0),
        };

        settings.save(&dir).expect("save");
        assert_eq!(Settings::load(&dir).expect("load"), settings);
    }

    #[test]
    fn a_corrupt_file_is_an_error_not_a_silent_reset() {
        let dir = temp_dir("corrupt");
        fs::write(Settings::path(&dir), "{ this is not json").expect("write");
        assert!(Settings::load(&dir).is_err());
    }

    #[test]
    fn a_file_from_a_newer_build_is_refused_rather_than_downgraded() {
        let dir = temp_dir("newer");
        fs::write(
            Settings::path(&dir),
            serde_json::json!({ "schema_version": SCHEMA_VERSION + 1 }).to_string(),
        )
        .expect("write");

        let error = Settings::load(&dir).expect_err("must refuse");
        assert!(
            error.to_string().contains("newer version"),
            "unexpected message: {error}"
        );
    }

    #[test]
    fn a_missing_field_is_an_error_rather_than_a_default() {
        // Losing a location to a silent default is worse than saying so.
        let dir = temp_dir("partial");
        fs::write(
            Settings::path(&dir),
            serde_json::json!({ "schema_version": SCHEMA_VERSION, "tray": { "subjects": [] } })
                .to_string(),
        )
        .expect("write");
        assert!(Settings::load(&dir).is_err());
    }

    /// A version 1 file must survive, with the meaning it had.
    ///
    /// A fresh install computes Rahu and Ketu the way a panchanga does.
    ///
    /// The mean node, not the true one. This is a default rather than a
    /// constant, so nothing else in the code asserts it - and it was shipped
    /// wrong once already, which is the reason for the test.
    #[test]
    fn a_new_install_uses_the_node_a_panchanga_uses() {
        assert_eq!(
            Settings::default().sidereal.node_type,
            NodeType::Mean,
            "the siddhantic Rahu is uniformly retrograde, which is the mean node"
        );
    }

    /// An existing file keeps the node type it names.
    ///
    /// Changing a default must not reach back into a document somebody already
    /// has: the true node moves Rahu by up to 1.6 degrees, which can put it in a
    /// different nakshatra, and a reading taken yesterday would silently stop
    /// agreeing with itself.
    #[test]
    fn changing_the_default_does_not_rewrite_a_file_that_names_one() {
        let dir = std::env::temp_dir().join("chandra-node-type-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");

        let mut written = Settings::default();
        written.sidereal.node_type = NodeType::True;
        written.save(&dir).expect("save");

        let read = Settings::load(&dir).expect("load");
        assert_eq!(
            read.sidereal.node_type,
            NodeType::True,
            "the file named the true node and must keep it"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// The moon survives becoming toggleable.
    ///
    /// Before schema 9 the moon was permanent and so was never written to
    /// `tray.subjects`; a file at 8 names only the grahas beside it. Making every
    /// calendar toggleable turned "absent from the list" from "permanent" into
    /// "switched off", so an upgrade that did nothing would have taken the moon
    /// out of the menu bar of every existing install.
    ///
    /// Checked against the old code: without the 8 -> 9 step this document
    /// migrates to an empty subject list and the assertion fails.
    #[test]
    fn the_moon_stays_in_the_menu_bar_across_schema_nine() {
        let dir = temp_dir("schema-nine");

        // A schema 8 document: the moon is not named because it could not be,
        // and the chart carries the switch it used to have.
        let document = serde_json::json!({
            "schema_version": 8,
            "location": { "mode": "automatic", "place": null, "elevation": null },
            "sidereal": { "ayanamsa": "lahiri", "node_type": "mean" },
            "calendar": { "month_system": "amanta", "ingress": "off" },
            "panchanga": { "yogas": false, "karanas": false, "muhurtas": false },
            "chart": { "tray": false, "format": "north", "numbered": false },
            "tray": { "subjects": ["shani"], "colour_mode": false },
            "appearance": { "scale": 1.0 }
        });
        fs::write(
            dir.join("settings.json"),
            serde_json::to_string(&document).expect("json"),
        )
        .expect("write");

        let settings = Settings::load(&dir).expect("load");
        assert_eq!(settings.schema_version, SCHEMA_VERSION);
        assert_eq!(
            settings.tray.subjects,
            vec![Graha::Chandra, Graha::Shani],
            "the moon was permanent, so it was on; it must stay on"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A field added to a schema version that already exists on disk.
    ///
    /// `numbered` was added to the chart block without bumping the version, on
    /// the reasoning that the step which created that block already wrote it.
    /// It did - but only for files that had not reached that version yet. Every
    /// install already at 7 had a two-field chart block, and the app refused to
    /// start with `missing field numbered`.
    ///
    /// This is the shape of that failure, so it cannot recur silently: a
    /// document at the previous version, without the new field, has to migrate.
    #[test]
    fn a_document_at_the_previous_version_gains_a_newly_required_field() {
        let dir = std::env::temp_dir().join("chandra-schema-7-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");

        let mut document = serde_json::to_value(Settings::default()).expect("serialise");
        document["schema_version"] = serde_json::Value::from(7u32);
        document["chart"] = serde_json::json!({ "tray": true, "format": "east" });
        fs::write(dir.join("settings.json"), document.to_string()).expect("write");

        let settings = Settings::load(&dir).expect("a version 7 document must migrate");
        assert_eq!(settings.schema_version, SCHEMA_VERSION);
        assert!(
            !settings.chart.numbered,
            "the default for a field nobody chose is the behaviour they already had"
        );
        assert_eq!(
            settings.chart.format,
            ChartFormat::East,
            "and the choices they did make survive"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// The degree grid arrives switched off, and a file written before it
    /// existed still loads.
    ///
    /// The same shape as the two tests above and for the same reason: schema 12
    /// is what every existing install is at, its chart block has four fields,
    /// and a fifth added without a migration is `missing field grid` on the
    /// next launch. Checked against the old code: without the 12 -> 13 step
    /// this document fails to load at all.
    ///
    /// It also pins the default. `animate` shipped *on*, because a reader who
    /// has never seen the chart move cannot ask for it; the grid ships off,
    /// because it draws the chart's own scaffolding over a chart the reader
    /// already has.
    #[test]
    fn the_degree_grid_arrives_off_and_a_schema_twelve_file_still_loads() {
        let dir = temp_dir("schema-thirteen");

        let mut document = serde_json::to_value(Settings::default()).expect("serialise");
        document["schema_version"] = serde_json::Value::from(12u32);
        document["chart"] = serde_json::json!({
            "format": "south",
            "numbered": true,
            "animate": false,
            "vargas": ["d1", "d9"]
        });
        fs::write(
            Settings::path(&dir),
            serde_json::to_string(&document).expect("json"),
        )
        .expect("write");

        let settings = Settings::load(&dir).expect("a version 12 document must migrate");
        assert_eq!(settings.schema_version, SCHEMA_VERSION);
        assert!(
            !settings.chart.grid,
            "the grid draws scaffolding; it cannot appear over a chart nobody asked to change"
        );
        assert_eq!(
            settings.chart.format,
            ChartFormat::South,
            "and every choice the reader did make survives"
        );
        assert!(settings.chart.numbered);
        assert!(!settings.chart.animate);
        assert_eq!(settings.chart.vargas, vec![Varga::D1, Varga::D9]);

        let _ = fs::remove_dir_all(&dir);
    }

    /// Version 1 carried the elevation inside the place; version 2 carries a
    /// correction beside it. `null` is what "no correction" is, so a document
    /// written by the previous build resolves to exactly the same observer.
    #[test]
    fn a_version_one_document_migrates_without_changing_what_it_meant() {
        let dir = temp_dir("migrate");
        fs::write(
            Settings::path(&dir),
            serde_json::json!({
                "schema_version": 1,
                "launch_at_login": true,
                "time_format": "hour24",
                "location": {
                    "mode": "manual",
                    "place": {
                        "label": "Bengaluru",
                        "zone": "Asia/Kolkata",
                        "latitude": 12.9716,
                        "longitude": 77.5946,
                        "elevation": 920.0
                    }
                },
                "sidereal": { "ayanamsa": "raman", "node_type": "mean" },
                "calendar": { "month_system": "amanta" },
                "tray": { "subjects": ["mangala"], "colour_mode": true }
            })
            .to_string(),
        )
        .expect("write");

        let settings = Settings::load(&dir).expect("migrates");
        assert_eq!(settings.schema_version, SCHEMA_VERSION);
        assert_eq!(
            settings.location.elevation,
            Some(920.0),
            "the elevation every rise and set was computed with comes across, or \
             it is lost the moment a different city is picked"
        );
        assert_eq!(
            settings.location.place.as_ref().map(|p| p.elevation),
            Some(Some(920.0)),
            "the place keeps the elevation it was written with"
        );
        assert_eq!(settings.sidereal.ayanamsa, Ayanamsa::Raman);
        assert_eq!(settings.sidereal.node_type, NodeType::Mean);
        // Mangala is what the file named. The moon is in front of it because
        // schema 8 and earlier could not name the moon - it was permanent - and
        // an upgrade that dropped it would switch off an item the user has been
        // looking at for as long as they have had the app.
        assert_eq!(settings.tray.subjects, vec![Graha::Chandra, Graha::Mangala]);
        assert!(settings.tray.colour_mode);

        // Saved back at the current version, and stable from there.
        settings.save(&dir).expect("save");
        assert_eq!(Settings::load(&dir).expect("reload"), settings);
    }

    #[test]
    fn saving_leaves_no_temporary_file_behind() {
        let dir = temp_dir("atomic");
        Settings::default().save(&dir).expect("save");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "left {leftovers:?}");
    }
}
