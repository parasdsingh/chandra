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

use chandra_ephemeris::{Ayanamsa, Graha, NodeType, SiderealConfig};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Bumped only when the shape changes in a way older files cannot satisfy.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub launch_at_login: bool,
    pub time_format: TimeFormat,
    pub location: LocationSetting,
    pub sidereal: SiderealSetting,
    pub tray: TraySetting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeFormat {
    /// Follow the operating system's 12 or 24 hour preference.
    System,
    Hour12,
    Hour24,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationSetting {
    pub mode: LocationMode,
    /// Present when `mode` is `Manual`, and also kept as the last known
    /// automatic result so a failed resolution does not lose the previous one.
    pub place: Option<PlaceSetting>,
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
            launch_at_login: false,
            time_format: TimeFormat::System,
            location: LocationSetting {
                mode: LocationMode::Automatic,
                place: None,
            },
            sidereal: SiderealSetting {
                ayanamsa: Ayanamsa::Lahiri,
                node_type: NodeType::True,
            },
            tray: TraySetting {
                // Only the moon, which is permanent and not listed here.
                subjects: Vec::new(),
                colour_mode: false,
            },
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
            launch_at_login: true,
            time_format: TimeFormat::Hour24,
            location: LocationSetting {
                mode: LocationMode::Manual,
                place: Some(PlaceSetting {
                    label: "Bengaluru".into(),
                    zone: "Asia/Kolkata".into(),
                    latitude: 12.9716,
                    longitude: 77.5946,
                    elevation: 920.0,
                }),
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
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| AppError::Settings(format!("{} is not valid JSON: {e}", path.display())))?;

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
        fs::write(&temporary, body)
            .map_err(|e| AppError::Settings(format!("cannot write {}: {e}", temporary.display())))?;
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
fn migrate(value: serde_json::Value, from: u32) -> Result<serde_json::Value> {
    match from {
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
        settings.time_format = TimeFormat::Hour24;
        settings.location = LocationSetting {
            mode: LocationMode::Manual,
            place: Some(PlaceSetting {
                label: "Bengaluru".into(),
                zone: "Asia/Kolkata".into(),
                latitude: 12.9716,
                longitude: 77.5946,
                elevation: 920.0,
            }),
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
            serde_json::json!({ "schema_version": SCHEMA_VERSION, "launch_at_login": true })
                .to_string(),
        )
        .expect("write");
        assert!(Settings::load(&dir).is_err());
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
