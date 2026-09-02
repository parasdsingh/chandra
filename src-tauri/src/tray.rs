//! Menu bar items.
//!
//! Every tray item is built in Rust and none is declared in `tauri.conf.json`.
//! Declaring one in both places produces two items on macOS, one of them inert
//! (tauri#8982, tauri#10912).

use chandra_almanac::lunar::MonthSystem;
use chandra_almanac::varga::Varga;
use chandra_almanac::{Snapshot, SnapshotGraha};
use chandra_ephemeris::Graha;
use chandra_glyph::{chart_icon, graha_icon, moon_icon, Tint};
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, Result};
use crate::panel;
use crate::state::AppState;

/// Identifier of Chandra's own item.
///
/// Named for the moon rather than for the graha because it draws the phase
/// rather than a glyph. It is a toggleable calendar like any other since D-030;
/// it is simply the one that is on by default.
pub const MOON_ID: &str = "chandra.moon";

/// A Lagna Kundali's item, one per division shown.
///
/// D1's keeps the bare id it has always had. It is the permanent one (D-030) and
/// the only chart item that existed before the divisions did.
fn chart_id(varga: Varga) -> String {
    if varga == Varga::D1 {
        return "chandra.chart".to_string();
    }
    format!("chandra.chart.{}", varga.key())
}

/// Backing scale for tray icons. Rendering at 2x and letting macOS map the
/// buffer onto the 22pt slot keeps the disc crisp on Retina, and downscales
/// acceptably on a 1x display.
const ICON_SCALE: u32 = 2;

fn tray_id(graha: Graha) -> String {
    // The moon keeps the id it has always had. Its item is the one that draws a
    // phase instead of a glyph, and `refresh_icons` finds it by this name.
    if graha == Graha::Chandra {
        return MOON_ID.to_string();
    }
    format!("chandra.{}", graha.key())
}

/// Builds the moon item and one item per enabled graha.
pub fn build(app: &AppHandle) -> Result<()> {
    let subjects = app.state::<AppState>().tray_subjects();

    // macOS puts each new status item to the *left* of the ones already there, so
    // creating in canonical order lays the row out backwards - and backwards is
    // what this row wants. Read left to right it runs Ketu first and Surya last,
    // the navagraha sequence reversed.
    //
    // Which way round matters because the two ends are not equivalent. When the
    // frontmost app has a long menu bar, macOS squeezes status items out **from
    // the left**, so the left end is the disposable one and the right end is the
    // one that survives. Running the sequence this way puts the two lights at
    // the end that survives and the nodes at the end that goes first, which is
    // the right way round to lose items.
    //
    // The chart is created first of all, so it sits furthest right. It is the
    // one item with no switch, so it is the one that has to be there - and it
    // used to be created last, which put it leftmost and made the feature that
    // had just been built the first thing macOS threw away (D-030).
    // D1 created first, so it sits furthest right - the end that survives when
    // macOS squeezes the row. The finer divisions run left from it in order, and
    // are the ones that can afford to go: D1 is the chart the others divide.
    //
    // Not reversed. macOS puts each new item to the *left* of the ones already
    // there, so creating in order lays them out right to left, which is what is
    // wanted here and the opposite of what the grahas need.
    for varga in app.state::<AppState>().chart_vargas() {
        build_item(app, chart_id(varga), panel::Subject::Chart(varga))?;
    }
    for graha in subjects {
        build_item(app, tray_id(graha), panel::Subject::Graha(graha))?;
    }

    refresh_icons(app)
}

fn build_item(app: &AppHandle, id: String, subject: panel::Subject) -> Result<()> {
    let handle = app.clone();
    TrayIconBuilder::with_id(id)
        // Left click opens the panel, so a menu on left click would swallow it.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |icon, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } => {
                    // The event carries the item's screen rectangle, so the
                    // panel is placed from that directly. Asking the window
                    // where it ended up after a positioning call instead races
                    // the window server, and reads a stale coordinate often
                    // enough to be visible.
                    panel::toggle(&handle, subject, rect);
                }
                // The pointer has arrived and the tooltip has not appeared yet -
                // macOS waits about a second before showing one. Rebuilding it
                // here is what makes it a reading of now rather than of whenever
                // the tray last happened to be redrawn.
                //
                // A failure is dropped: the item keeps the text it had, which is
                // at most an hour old, and there is nowhere to report a tooltip
                // to anyway.
                TrayIconEvent::Enter { .. } => {
                    refresh_tooltip(&handle, icon.id().clone(), subject);
                }
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| {
            AppError::Engine(format!(
                "cannot create the {} tray item: {e}",
                subject.key()
            ))
        })?;
    Ok(())
}

/// Tears down every tray item and builds the current set again.
///
/// Rebuilding wholesale rather than diffing: the set changes only when the user
/// toggles a graha in settings, and a diff would be more code than it saves.
pub fn rebuild(app: &AppHandle) -> Result<()> {
    for graha in Graha::ALL {
        app.remove_tray_by_id(&tray_id(graha));
    }
    for varga in Varga::ALL {
        app.remove_tray_by_id(&chart_id(varga));
    }
    app.remove_tray_by_id(MOON_ID);
    build(app)
}

/// Redraws every tray icon from the current instant and settings.
pub fn refresh_icons(app: &AppHandle) -> Result<()> {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let location = state.location();
    let subjects = state.tray_subjects();

    let tint = if settings.tray.colour_mode {
        // The panel's primary text colour, so a coloured tray icon matches the
        // panel it opens.
        Tint::Colour {
            r: 237,
            g: 237,
            b: 239,
        }
    } else {
        Tint::Template
    };
    // A template image is inverted by macOS to suit the menu bar; a fixed colour
    // must not be, or it would be inverted into its opposite.
    let is_template = !settings.tray.colour_mode;

    let now = jiff::Timestamp::now().as_millisecond();
    let snapshot = state.almanac.now(now, &subjects).map_err(AppError::from)?;

    if let Some(item) = app.tray_by_id(MOON_ID) {
        let icon = moon_icon(
            snapshot.illumination,
            snapshot.is_waxing,
            location.latitude < 0.0,
            ICON_SCALE,
            tint,
        )
        .map_err(|e| AppError::Engine(format!("cannot draw the moon icon: {e}")))?;

        item.set_icon_with_as_template(Some(to_image(&icon)), is_template)
            .map_err(|e| AppError::Engine(format!("cannot set the moon icon: {e}")))?;
        item.set_tooltip(Some(moon_tooltip(
            &snapshot,
            settings.calendar.month_system,
        )))
        .map_err(|e| AppError::Engine(format!("cannot set the moon tooltip: {e}")))?;
    }

    for varga in state.chart_vargas() {
        let Some(item) = app.tray_by_id(&chart_id(varga)) else {
            continue;
        };
        // The division is drawn into the mark. Several charts can be in the row
        // at once and they are otherwise the same shape, so without the number
        // the reader has a line of identical icons and no way to tell which is
        // which but to hover each one.
        let icon = chart_icon(varga.division(), ICON_SCALE, tint)
            .map_err(|e| AppError::Engine(format!("cannot draw the chart icon: {e}")))?;
        item.set_icon_with_as_template(Some(to_image(&icon)), is_template)
            .map_err(|e| AppError::Engine(format!("cannot set the chart icon: {e}")))?;
        // The tooltip is rebuilt on hover, which is what makes the lagna in it
        // current to the second. This is the text before the first hover.
        item.set_tooltip(Some(varga.label()))
            .map_err(|e| AppError::Engine(format!("cannot set the chart tooltip: {e}")))?;
    }

    for graha in subjects {
        // The moon's item is drawn above, as a phase rather than as a glyph. It
        // is in `subjects` now, so without this it would be drawn twice and the
        // glyph would win.
        if graha == Graha::Chandra {
            continue;
        }
        let Some(item) = app.tray_by_id(&tray_id(graha)) else {
            continue;
        };
        let position = snapshot.grahas.iter().find(|p| p.graha == graha);

        // Retrograde is the one state the menu bar carries. Everything else a
        // graha can be doing is named in the calendar, in words.
        let retrograde = position.is_some_and(|p| p.retrograde);
        let icon = graha_icon(graha, ICON_SCALE, tint, retrograde)
            .map_err(|e| AppError::Engine(format!("cannot draw the {} icon: {e}", graha.name())))?;
        item.set_icon_with_as_template(Some(to_image(&icon)), is_template)
            .map_err(|e| AppError::Engine(format!("cannot set the {} icon: {e}", graha.name())))?;

        item.set_tooltip(Some(graha_tooltip(graha, position)))
            .map_err(|e| {
                AppError::Engine(format!("cannot set the {} tooltip: {e}", graha.name()))
            })?;
    }

    Ok(())
}

/// Rebuilds one item's tooltip from this instant.
///
/// Off the main thread. This runs from the tray icon's event callback, which
/// `tao` delivers on the main thread - and `Engine`'s own contract is that it is
/// called "from a blocking pool, never from a UI thread" (`engine.rs`). An
/// ephemeris call there takes the engine mutex and can be made to wait behind a
/// month being assembled, which would stall the menu bar and every window with
/// it. The first version of this did exactly that.
///
/// macOS waits about a second before showing a tooltip, so there is room to
/// compute one in between and still have it be the text that appears. Setting it
/// goes back to the main thread, because a status item is an AppKit object.
///
/// Only the tooltip, and only this item. Redrawing the icon here would put a
/// rasterise in the same window for a change smaller than a pixel.
fn refresh_tooltip(app: &AppHandle, id: tauri::tray::TrayIconId, subject: panel::Subject) {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let system = state.settings().calendar.month_system;

        // Asked for only the subject this item carries. The pointer is over one
        // item and the other tooltips are not about to be read.
        let subjects = match subject {
            panel::Subject::Graha(Graha::Chandra) | panel::Subject::Chart(_) => Vec::new(),
            panel::Subject::Graha(graha) => vec![graha],
        };
        let now = jiff::Timestamp::now().as_millisecond();
        let Ok(snapshot) = state.almanac.now(now, &subjects) else {
            // The item keeps the text it had, which is at most an hour old.
            // There is nowhere to report a tooltip failure to.
            return;
        };

        let tooltip = match subject {
            panel::Subject::Graha(Graha::Chandra) => moon_tooltip(&snapshot, system),
            panel::Subject::Graha(graha) => graha_tooltip(graha, snapshot.grahas.first()),
            // The lagna, which is the one thing on this chart that changes fast
            // enough to be worth a hover - about a degree every four minutes.
            panel::Subject::Chart(varga) => chart_tooltip(&handle, varga),
        };

        let for_main = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            if let Some(item) = for_main.tray_by_id(&id) {
                let _ = item.set_tooltip(Some(tooltip));
            }
        });
    });
}

/// What the moon item says on hover, in the calendar that is in force.
///
/// The percentage lit is not here. It is the answer to "how much of the disc is
/// showing", which is a solar-calendar question and is already drawn - the icon
/// beside the pointer is that number. In a lunar month the useful fact is which
/// tithi it is, because that is what the day is called and what a transit is
/// read against.
fn moon_tooltip(snapshot: &Snapshot, system: MonthSystem) -> String {
    if system.is_lunar() {
        format!("Chandra - {}", snapshot.tithi)
    } else {
        format!("Chandra - {}", snapshot.phase.label())
    }
}

/// What the chart item says on hover: what is rising, and how far into it.
fn chart_tooltip(app: &AppHandle, varga: Varga) -> String {
    let state = app.state::<AppState>();
    let now = jiff::Timestamp::now().as_millisecond();
    let place = state.location().label;
    // The item's own division, not a setting. Several are in the row at once, so
    // reading one from settings would give every one of them the same answer.
    match state.almanac.chakra(now, &place, varga) {
        Ok(chart) => {
            let (degrees, minutes, _) = chart.lagna.degrees_in_rashi;
            format!(
                "{} - {} {degrees}\u{00b0}{minutes:02}\u{2032}",
                varga.label(),
                chart.lagna.name
            )
        }
        // The item still names itself. A tooltip is not the place to report that
        // an ephemeris call failed.
        Err(_) => varga.label().to_string(),
    }
}

/// What a graha item says on hover.
fn graha_tooltip(graha: Graha, position: Option<&SnapshotGraha>) -> String {
    match position {
        Some(p) => format!(
            "{} - {}{}",
            graha.name(),
            p.rashi.name(),
            if p.retrograde { ", retrograde" } else { "" }
        ),
        None => graha.name().to_string(),
    }
}

fn to_image(icon: &chandra_glyph::Icon) -> Image<'_> {
    Image::new(&icon.rgba, icon.width, icon.height)
}
