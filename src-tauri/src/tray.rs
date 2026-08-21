//! Menu bar items.
//!
//! Every tray item is built in Rust and none is declared in `tauri.conf.json`.
//! Declaring one in both places produces two items on macOS, one of them inert
//! (tauri#8982, tauri#10912).

use chandra_ephemeris::Graha;
use chandra_glyph::{graha_icon, moon_icon, Tint};
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, Result};
use crate::panel;
use crate::state::AppState;

/// Identifier of the permanent moon item.
///
/// The moon item *is* Chandra's item; enabling Chandra in settings does not add
/// a second one (`docs/DECISIONS.md` D-019).
pub const MOON_ID: &str = "chandra.moon";

/// Backing scale for tray icons. Rendering at 2x and letting macOS map the
/// buffer onto the 22pt slot keeps the disc crisp on Retina, and downscales
/// acceptably on a 1x display.
const ICON_SCALE: u32 = 2;

fn tray_id(graha: Graha) -> String {
    format!("chandra.{}", graha.key())
}

/// Builds the moon item and one item per enabled graha.
pub fn build(app: &AppHandle) -> Result<()> {
    let subjects = app.state::<AppState>().tray_subjects();

    build_item(app, MOON_ID.to_string(), Graha::Chandra)?;
    for graha in subjects {
        build_item(app, tray_id(graha), graha)?;
    }

    refresh_icons(app)
}

fn build_item(app: &AppHandle, id: String, subject: Graha) -> Result<()> {
    let handle = app.clone();
    TrayIconBuilder::with_id(id)
        // Left click opens the panel, so a menu on left click would swallow it.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |_icon, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                // The event carries the item's screen rectangle, so the panel is
                // placed from that directly. Asking the window where it ended up
                // after a positioning call instead races the window server, and
                // reads a stale coordinate often enough to be visible.
                panel::toggle(&handle, subject, rect);
            }
        })
        .build(app)
        .map_err(|e| {
            AppError::Engine(format!(
                "cannot create the {} tray item: {e}",
                subject.name()
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
        item.set_tooltip(Some(format!(
            "Chandra - {}, {:.0}% lit",
            snapshot.phase.label(),
            snapshot.illumination * 100.0
        )))
        .map_err(|e| AppError::Engine(format!("cannot set the moon tooltip: {e}")))?;
    }

    for graha in subjects {
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

        let tooltip = match position {
            Some(p) => format!(
                "{} - {}{}",
                graha.name(),
                p.rashi.name(),
                if p.retrograde { ", retrograde" } else { "" }
            ),
            None => graha.name().to_string(),
        };
        item.set_tooltip(Some(tooltip)).map_err(|e| {
            AppError::Engine(format!("cannot set the {} tooltip: {e}", graha.name()))
        })?;
    }

    Ok(())
}

fn to_image(icon: &chandra_glyph::Icon) -> Image<'_> {
    Image::new(&icon.rgba, icon.width, icon.height)
}
