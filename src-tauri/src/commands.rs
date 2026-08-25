//! The IPC surface.
//!
//! Every command is async and moves its work onto a blocking thread: the
//! ephemeris engine is a mutex-guarded C library (`docs/DECISIONS.md` D-005) and
//! blocking Tauri's async runtime on it would stall every other command.

use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::month::{DayDetail, GrahaMonth, MoonMonth};
use chandra_almanac::time::DateKey;
use chandra_almanac::Snapshot;
use chandra_ephemeris::{Ayanamsa, Graha, NodeType};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, Result};
use crate::location::Resolved;
use crate::settings::Settings;
use crate::state::AppState;
use crate::{panel, tray};

/// Everything the front end needs in its first frame, in one round trip.
///
/// A single call rather than five, because the panel has 120ms before it must be
/// on screen and five sequential IPC round trips would spend a visible part of
/// that budget on nothing.
#[derive(Debug, Serialize)]
pub struct Bootstrap {
    pub settings: Settings,
    pub location: Resolved,
    /// Which subject the panel is currently showing.
    pub subject: Graha,
    pub subjects: Vec<Graha>,
    /// Whether the system's popover material is behind the panel. The panel is
    /// a scrim over that material, so where it is absent the front end has to
    /// paint an opaque ground instead of letting the desktop through.
    pub panel_material: bool,
    pub library_version: String,
    /// Names for the settings pickers, so the front end holds no duplicate list
    /// that could fall out of step with the ephemeris.
    pub ayanamsas: Vec<Choice>,
    pub node_types: Vec<Choice>,
    pub month_systems: Vec<Choice>,
    pub grahas: Vec<GrahaInfo>,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub key: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Serialize)]
pub struct GrahaInfo {
    pub key: &'static str,
    pub name: &'static str,
    pub english: &'static str,
    /// SVG path data on a 24 x 24 grid, served from `chandra-glyph` rather than
    /// duplicated in the front end. The menu bar and the panel then draw the
    /// same nine glyphs from one source, and cannot drift apart.
    pub path: &'static str,
    pub filled: bool,
    pub stroke_width: f32,
}

#[tauri::command]
pub async fn bootstrap(app: AppHandle, state: State<'_, AppState>) -> Result<Bootstrap> {
    Ok(Bootstrap {
        settings: state.settings(),
        location: state.location(),
        subject: panel::subject_or_default(&app),
        subjects: state.tray_subjects(),
        panel_material: panel::has_material(&app),
        library_version: state.almanac.library_version().map_err(AppError::from)?,
        ayanamsas: Ayanamsa::ALL
            .into_iter()
            .map(|a| Choice {
                key: a.key(),
                label: a.label(),
            })
            .collect(),
        node_types: [NodeType::True, NodeType::Mean]
            .into_iter()
            .map(|n| Choice {
                key: n.key(),
                label: n.label(),
            })
            .collect(),
        month_systems: MonthSystem::ALL
            .into_iter()
            .map(|s| Choice {
                key: s.key(),
                label: s.label(),
            })
            .collect(),
        grahas: Graha::ALL
            .into_iter()
            .map(|g| {
                let (path, ink) = chandra_glyph::glyphs::glyph(g);
                GrahaInfo {
                    key: g.key(),
                    name: g.name(),
                    english: g.english(),
                    path,
                    filled: ink == chandra_glyph::glyphs::Ink::Fill,
                    stroke_width: chandra_glyph::glyphs::STROKE_WIDTH,
                }
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn moon_month(
    app: AppHandle,
    anchor_unix_ms: i64,
    offset: i32,
    first_weekday: u8,
) -> Result<MoonMonth> {
    blocking(app, move |state| {
        let cursor = state.cursor(anchor_unix_ms, offset, first_weekday);
        state.almanac.moon_month(cursor).map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn graha_month(
    app: AppHandle,
    graha: Graha,
    anchor_unix_ms: i64,
    offset: i32,
    first_weekday: u8,
) -> Result<GrahaMonth> {
    blocking(app, move |state| {
        let cursor = state.cursor(anchor_unix_ms, offset, first_weekday);
        state
            .almanac
            .graha_month(graha, cursor)
            .map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn day_detail(
    app: AppHandle,
    graha: Graha,
    year: i16,
    month: i8,
    day: i8,
) -> Result<DayDetail> {
    blocking(app, move |state| {
        let date = DateKey::new(year, month, day).map_err(AppError::from)?;
        let system = state.settings().calendar.month_system;
        state
            .almanac
            .day_detail(graha, date, system)
            .map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn snapshot(app: AppHandle, unix_ms: i64) -> Result<Snapshot> {
    blocking(app, move |state| {
        let subjects = state.tray_subjects();
        state
            .almanac
            .now(unix_ms, &subjects)
            .map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: Settings) -> Result<Bootstrap> {
    // Off the async runtime like every other command: `apply` takes the engine
    // mutex and writes and renames a file, and the module's own contract is that
    // none of that happens on the runtime's threads.
    let applied = blocking(app.clone(), move |state| state.apply(settings)).await?;

    // Status items are AppKit objects: creating, removing or redrawing one off
    // the main thread crashes the process. Commands run on the async runtime,
    // so the work is dispatched rather than called directly.
    if applied.tray_changed || applied.icons_changed {
        let handle = app.clone();
        let rebuild = applied.tray_changed;
        app.run_on_main_thread(move || {
            let outcome = if rebuild {
                tray::rebuild(&handle)
            } else {
                tray::refresh_icons(&handle)
            };
            if let Err(error) = outcome {
                // The settings themselves are already saved; a menu bar that
                // did not update is worth reporting but not worth failing over.
                eprintln!("chandra: could not update the menu bar: {error}");
            }
        })
        .map_err(|error| AppError::Engine(format!("cannot reach the main thread: {error}")))?;
    }

    let state = app.state::<AppState>();
    bootstrap(app.clone(), state).await
}

#[tauri::command]
pub async fn search_cities(query: String, limit: usize) -> Result<Vec<chandra_geo::Place>> {
    Ok(chandra_geo::search(&query, limit.min(50))
        .into_iter()
        .cloned()
        .collect())
}

/// Asks macOS for the device's coordinates and records them if it answers.
///
/// Returns the location in force after the attempt, which may be unchanged: a
/// denial is a normal outcome, not an error, and the app keeps working on the
/// timezone fallback.
#[tauri::command]
pub async fn request_device_location(app: AppHandle) -> Result<Resolved> {
    panel::request_device_location(&app).await;
    Ok(app.state::<AppState>().location())
}

#[tauri::command]
pub async fn close_panel(app: AppHandle) -> Result<()> {
    panel::hide(&app);
    Ok(())
}

/// Runs `work` on a blocking thread with access to application state.
async fn blocking<T, F>(app: AppHandle, work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&AppState) -> Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || work(&app.state::<AppState>()))
        .await
        .map_err(|e| AppError::Engine(format!("calculation task failed: {e}")))?
}
