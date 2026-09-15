//! The IPC surface.
//!
//! Every command is async and moves its work onto a blocking thread: the
//! ephemeris engine is a mutex-guarded C library (`docs/DECISIONS.md` D-005) and
//! blocking Tauri's async runtime on it would stall every other command.

use chandra_almanac::chakra::Chakra;
use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::month::{DayDetail, GrahaMonth, MonthIndex, MoonMonth};
use chandra_almanac::time::DateKey;
use chandra_almanac::varga::Varga;
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
    /// Which subject the panel is showing: a graha's key, or `chart`.
    pub subject: String,
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
    /// The divisional charts on offer. Served rather than written out in the
    /// front end so nothing here can drift from `Varga`'s own answers.
    pub vargas: Vec<VargaInfo>,
    /// Why the stored settings were not used, if they were not.
    ///
    /// A settings file that cannot be read is a real event the reader has to be
    /// told about: it means the choices on screen are the defaults and not
    /// theirs. The file is left exactly as it is - resetting it silently would
    /// destroy the only copy of what they had chosen - so the app runs on
    /// defaults until they either fix the file or change a setting, which
    /// overwrites it deliberately.
    pub settings_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub key: &'static str,
    pub label: &'static str,
}

/// A divisional chart, as the front end needs it.
///
/// More than a `Choice` because three surfaces need different parts of it: the
/// menu bar draws the number, the panel's title names the division, and the
/// settings list shows both.
#[derive(Debug, Serialize)]
pub struct VargaInfo {
    pub key: &'static str,
    /// `D9 · Navamsa`, for a list.
    pub label: &'static str,
    /// `Navamsa`, for a title that already has room for little else.
    pub name: &'static str,
    /// The number in the name, which is what the menu bar icon draws.
    pub division: u32,
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
    /// `x y width height`, centred on the glyph's own ink rather than on the
    /// design grid. Several glyphs are drawn off-centre in the grid, so a ring
    /// concentric with the box was not concentric with the symbol.
    pub view_box: [f32; 4],
}

#[tauri::command]
pub async fn bootstrap(app: AppHandle, state: State<'_, AppState>) -> Result<Bootstrap> {
    Ok(Bootstrap {
        // Clamped here, so the number the page scales itself by and the one the
        // window is sized by are the same number. Serialised raw, a hand-edited
        // scale of 4 gave a transform of 4 inside a window built at 1.4.
        settings: {
            let mut settings = state.settings();
            settings.appearance.scale = settings.appearance.clamped();
            settings
        },
        location: state.location(),
        settings_error: state.settings_error(),
        subject: panel::subject_or_default(&app).key().to_string(),
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
        // The default first, which is the mean node: it is what a panchanga
        // uses, and a list whose recommended entry is second reads as if the
        // first one were.
        node_types: [NodeType::Mean, NodeType::True]
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
        grahas: graha_info(),
        vargas: chandra_almanac::varga::Varga::ALL
            .into_iter()
            .map(|v| VargaInfo {
                key: v.key(),
                label: v.label(),
                name: v.name(),
                division: v.division(),
            })
            .collect(),
    })
}

/// Every graha's symbol, as the panel receives it.
///
/// Its own function because the visual harness needs the same list and cannot
/// call `bootstrap`, which wants an `AppHandle` and a `State`. Building it there
/// instead - as an untyped `json!` literal, which is what it was - meant the
/// harness silently stopped matching `GrahaInfo` the moment a field was added:
/// `view_box` arrived, the fixture kept the old six keys, and every glyph in the
/// harness threw on a `view_box` that was not there. One typed source cannot
/// drift, because the compiler will not let it.
pub fn graha_info() -> Vec<GrahaInfo> {
    Graha::ALL
        .into_iter()
        .map(|g| {
            let (path, ink) = chandra_glyph::glyphs::glyph(g);
            let (x, y, width, height) = chandra_glyph::glyphs::view_box(g);
            GrahaInfo {
                key: g.key(),
                name: g.name(),
                english: g.english(),
                path,
                filled: ink == chandra_glyph::glyphs::Ink::Fill,
                stroke_width: chandra_glyph::glyphs::STROKE_WIDTH,
                view_box: [x, y, width, height],
            }
        })
        .collect()
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

/// The months of the year a cursor lands in, for the jump overlay.
///
/// Its own command rather than a field on `moon_month`: the overlay is opened
/// deliberately and rarely, and a lunar year costs up to fourteen syzygy
/// searches to enumerate. Every month view would have paid that for a panel
/// almost nobody opens.
#[tauri::command]
pub async fn month_index(
    app: AppHandle,
    anchor_unix_ms: i64,
    offset: i32,
    first_weekday: u8,
) -> Result<MonthIndex> {
    blocking(app, move |state| {
        let cursor = state.cursor(anchor_unix_ms, offset, first_weekday);
        state.almanac.month_index(cursor).map_err(AppError::from)
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
        // Read here rather than sent by the front end: which limbs are on is a
        // property of the configuration, not of the request, so a day cannot be
        // asked for with a set of limbs the settings pane does not agree with.
        let options = state.settings().panchanga.into();
        state
            .almanac
            .day_detail(graha, date, options)
            .map_err(AppError::from)
    })
    .await
}

/// The Lagna Kundali at an instant.
///
/// Takes the instant from the caller rather than reading the clock here, for the
/// same reason `snapshot` does: the front end decides how often it wants a new
/// one, and a command that read its own clock could not be asked for the same
/// moment twice.
#[tauri::command]
pub async fn chakra(app: AppHandle, unix_ms: i64, varga: String) -> Result<Chakra> {
    blocking(app, move |state| {
        // The label comes from the resolution chain, which is the only layer
        // that knows what the coordinates are called.
        let place = state.location().label;
        // Named by the caller, because several charts can be open in turn and the
        // panel knows which item was clicked. An unknown key is the rashi chart:
        // a division the back end does not have is a front end out of step, and
        // the chart it has always drawn is the safe answer.
        let varga = Varga::ALL
            .into_iter()
            .find(|v| v.key() == varga)
            .unwrap_or(Varga::D1);
        state
            .almanac
            .chakra(unix_ms, &place, varga)
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

    // A window resize is AppKit's business too, and for the same reason must be
    // dispatched rather than called from the runtime this command runs on.
    if applied.scale_changed {
        let handle = app.clone();
        let scale = handle.state::<AppState>().settings().appearance.clamped();
        let _ = app.run_on_main_thread(move || panel::apply_scale(&handle, scale));
    }

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

/// What to call a pair of coordinates.
///
/// For coordinates typed by hand, which no search produced and so which arrive
/// with no name. The nearest city is a **label only** - the timezone is not
/// taken from it, and cannot be: a zone boundary is political and is not
/// recoverable from a point at any precision (D-031).
///
/// `None` for coordinates that are not coordinates. A NaN or a value off the
/// globe has no nearest anything, and answering with the first row of the table
/// would be as confident as a real answer.
#[tauri::command]
pub async fn nearest_city(latitude: f64, longitude: f64) -> Result<Option<chandra_geo::Place>> {
    Ok(chandra_geo::nearest_place(latitude, longitude).cloned())
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
