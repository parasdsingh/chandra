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

use chandra_almanac::lunar::MonthSystem;
use chandra_ephemeris::{Ayanamsa, Graha, NodeType, SiderealConfig};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Bumped only when the shape changes in a way older files cannot satisfy.
pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub location: LocationSetting,
    pub sidereal: SiderealSetting,
    pub calendar: CalendarSetting,
    pub tray: TraySetting,
    pub appearance: AppearanceSetting,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarSetting {
    /// Whether months run Gregorian, new moon to new moon, or full moon to full
    /// moon.
    pub month_system: MonthSystem,
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
    /// Metres above sea level. Affects rise and set by roughly four minutes at
    /// 900 m, so it is a real input rather than a decoration.
    pub elevation: f64,
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
    /// Grahas with their own menu bar item, besides the permanent moon.
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
                node_type: NodeType::True,
            },
            calendar: CalendarSetting {
                month_system: MonthSystem::Solar,
            },
            tray: TraySetting {
                // Only the moon, which is permanent and not listed here.
                subjects: Vec::new(),
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
                    elevation: 920.0,
                }),
                elevation: Some(940.0),
            },
            calendar: CalendarSetting {
                month_system: MonthSystem::Amanta,
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
        assert_eq!(settings.sidereal.node_type, NodeType::True);
        assert!(settings.tray.subjects.is_empty());
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
                elevation: 920.0,
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
            Some(920.0),
            "the place keeps the elevation it was written with"
        );
        assert_eq!(settings.sidereal.ayanamsa, Ayanamsa::Raman);
        assert_eq!(settings.sidereal.node_type, NodeType::Mean);
        assert_eq!(settings.tray.subjects, vec![Graha::Mangala]);
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
