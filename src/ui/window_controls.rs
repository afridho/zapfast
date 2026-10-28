//! macOS-style traffic lights ZapFast draws on Linux when it draws its own
//! title bar.
//!
//! On Linux the custom title bar normally draws GNOME's round caption buttons
//! through fastframe-titlebar. This fork prefers the macOS look: three
//! coloured circles (close, minimize, zoom) at the window's top-left corner,
//! each showing its glyph on hover, as macOS and Zen Browser draw them. The
//! buttons send the same viewport commands fastframe-titlebar's do, so the
//! window behaves identically; only the drawing differs.
//!
//! The room the lights take is reported through [`crate::theme::caption_footprint`]
//! as a left-side footprint, so every header's `caption_room` keeps clear of
//! them with the same layout machinery macOS uses.

use egui::{Align2, Color32, Sense, Stroke, Vec2, pos2, vec2};

use crate::theme::{self, Icon};

/// Width the lights take at the start of the title bar, in egui points: three
/// buttons from 16 points in, 20 apart, and a right margin, matching AppKit's
/// [`fastframe_titlebar::TRAFFIC_LIGHTS_WIDTH`].
pub const WIDTH: f32 = 84.0;

/// Left edge of the close button, in points.
const FIRST: f32 = 16.0;
/// Distance between the buttons' centres.
const SPACING: f32 = 20.0;
/// Each circle's diameter.
const DIAMETER: f32 = 12.0;

/// The three lights, left to right.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Light {
    Close,
    Minimize,
    Zoom,
}

impl Light {
    /// The light's colour when the window is focused, as macOS draws them.
    fn color(self) -> Color32 {
        match self {
            Self::Close => Color32::from_rgb(0xff, 0x5f, 0x57),
            Self::Minimize => Color32::from_rgb(0xfe, 0xbc, 0x2e),
            Self::Zoom => Color32::from_rgb(0x28, 0xc8, 0x40),
        }
    }

    /// A slightly darker rim, so a light reads on a light header too.
    fn rim(self) -> Color32 {
        match self {
            Self::Close => Color32::from_rgb(0xe0, 0x44, 0x3e),
            Self::Minimize => Color32::from_rgb(0xdd, 0xa0, 0x1f),
            Self::Zoom => Color32::from_rgb(0x1a, 0xac, 0x2e),
        }
    }
}

/// Draws the traffic lights over the window's top-left corner, above every
/// panel, and acts on their clicks. Call it each frame while ZapFast draws
/// its own title bar; it is laid out at zoom 1 so it keeps its native size,
/// as macOS's lights do.
pub fn controls(ctx: &egui::Context, palette: &crate::theme::Palette, locale: crate::i18n::Locale) {
    let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
    // The lights do not scale with the interface zoom, matching the macOS
    // traffic lights' fixed size; a zoomed interface keeps them unchanged.
    let zoom = ctx.zoom_factor();
    let top = (theme::TOP_BAR_HEIGHT / 2.0) / zoom;
    egui::Area::new(egui::Id::new("zapfast-traffic-lights"))
        .anchor(Align2::LEFT_TOP, vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            // Lay out at the native scale regardless of the interface zoom.
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let scale = 1.0 / zoom;
            let diameter = DIAMETER * scale;
            let (rect, _) = ui.allocate_exact_size(
                vec2(WIDTH * scale, theme::TOP_BAR_HEIGHT * scale),
                Sense::hover(),
            );
            let hovered_any = ui.rect_contains_pointer(rect);
            for (index, light) in [Light::Close, Light::Minimize, Light::Zoom]
                .into_iter()
                .enumerate()
            {
                let center = pos2(
                    rect.left() + (FIRST + index as f32 * SPACING) * scale,
                    rect.top() + top * scale,
                );
                button(
                    ui,
                    light,
                    center,
                    diameter,
                    hovered_any,
                    maximized,
                    palette,
                    locale,
                );
            }
        });
    ctx.move_to_top(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("zapfast-traffic-lights"),
    ));
}

#[expect(clippy::too_many_arguments)]
fn button(
    ui: &mut egui::Ui,
    light: Light,
    center: egui::Pos2,
    diameter: f32,
    hovered_any: bool,
    maximized: bool,
    palette: &crate::theme::Palette,
    locale: crate::i18n::Locale,
) {
    let radius = diameter / 2.0;
    let rect = egui::Rect::from_center_size(center, Vec2::splat(diameter));
    let id = ui.id().with(match light {
        Light::Close => "close",
        Light::Minimize => "minimize",
        Light::Zoom => "zoom",
    });
    let response = ui.interact(rect, id, Sense::click());
    ui.painter()
        .circle(center, radius, light.color(), Stroke::new(1.0, light.rim()));
    // The glyph appears while the pointer is anywhere over the group, as on
    // macOS, and darkens on the hovered light itself.
    if hovered_any {
        let (icon, _) = glyph(light, maximized, locale);
        let alpha = if response.hovered() { 180 } else { 120 };
        let tint = Color32::from_rgba_unmultiplied(0, 0, 0, alpha);
        icon.image(tint, diameter)
            .fit_to_exact_size(Vec2::splat(diameter))
            .paint_at(ui, rect);
    }
    // A pressed light is a touch darker, so the press reads.
    if response.is_pointer_button_down_on() {
        ui.painter()
            .circle_filled(center, radius, Color32::from_rgba_unmultiplied(0, 0, 0, 40));
    }
    let _ = palette;
    let (_, label) = glyph(light, maximized, locale);
    let response = response.on_hover_text(label);
    if response.clicked() {
        let command = match light {
            Light::Close => egui::ViewportCommand::Close,
            Light::Minimize => egui::ViewportCommand::Minimized(true),
            Light::Zoom => egui::ViewportCommand::Maximized(!maximized),
        };
        ui.ctx().send_viewport_cmd(command);
    }
}

/// The glyph and accessible label for a light.
fn glyph(
    light: Light,
    maximized: bool,
    locale: crate::i18n::Locale,
) -> (Icon, std::borrow::Cow<'static, str>) {
    match light {
        Light::Close => (Icon::TrafficClose, crate::i18n::gettext(locale, "Close")),
        Light::Minimize => (
            Icon::TrafficMinimize,
            crate::i18n::gettext(locale, "Minimize"),
        ),
        Light::Zoom if maximized => (
            Icon::TrafficRestore,
            crate::i18n::gettext(locale, "Restore"),
        ),
        Light::Zoom => (
            Icon::TrafficMaximize,
            crate::i18n::gettext(locale, "Maximize"),
        ),
    }
}

/// Whether ZapFast draws the macOS-style traffic lights itself, instead of
/// fastframe-titlebar's GNOME buttons: on Linux while its own title bar is
/// visible (enabled and not in full screen).
pub fn active(ctx: &egui::Context) -> bool {
    cfg!(target_os = "linux") && fastframe_titlebar::visible(ctx)
}

/// Corner radius of the frameless window, in points: GNOME's rounded-corner
/// and liquid-glass extensions round windows by 20 by default.
pub const CORNER_RADIUS: f32 = 20.0;

/// Whether the frameless Linux window gets a transparent surface so its
/// corners can be cut round. Without one a compositor that rounds windows
/// (GNOME's rounded-corner or liquid-glass extensions) still shows the square
/// surface behind its rounded glass.
pub fn rounds_corners() -> bool {
    cfg!(target_os = "linux") && fastframe_titlebar::enabled()
}

/// How far in from the side the round corner starts on pixel row `row`
/// (0 at the corner's outer edge) for a radius of `radius` pixels.
fn corner_inset(radius: f32, row: u32) -> u32 {
    let dy = radius - (row as f32 + 0.5);
    let inside = (radius * radius - dy * dy).max(0.0).sqrt();
    (radius - inside).round().max(0.0) as u32
}

/// Clears the window's four corners to transparent, round, after everything
/// else has painted, so the desktop shows through them. Does nothing on an
/// opaque window, or when the window fills the screen.
pub fn cut_corners(ctx: &egui::Context) {
    if !rounds_corners()
        || !crate::backdrop::window_transparent()
        || ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))
    {
        return;
    }
    let rect = ctx.content_rect();
    let layer = egui::LayerId::new(egui::Order::Debug, egui::Id::new("zapfast-round-corners"));
    let callback = eframe::egui_glow::CallbackFn::new(|info, painter| {
        use eframe::glow::HasContext as _;
        let [width, height] = info.screen_size_px;
        let radius = (CORNER_RADIUS * info.pixels_per_point).round();
        let rows = (radius as u32).min(height / 2);
        let gl = painter.gl();
        // SAFETY: called by egui_glow on the thread owning its context,
        // with the window's framebuffer bound; egui restores its state after.
        unsafe {
            gl.enable(eframe::glow::SCISSOR_TEST);
            gl.color_mask(true, true, true, true);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            for row in 0..rows {
                let inset = corner_inset(radius, row).min(width / 2);
                if inset == 0 {
                    continue;
                }
                let (inset, row) = (inset as i32, row as i32);
                let (w, h) = (width as i32, height as i32);
                // GL counts rows from the bottom.
                for (x, y) in [
                    (0, h - 1 - row),
                    (w - inset, h - 1 - row),
                    (0, row),
                    (w - inset, row),
                ] {
                    gl.scissor(x, y, inset, 1);
                    gl.clear(eframe::glow::COLOR_BUFFER_BIT);
                }
            }
        }
    });
    ctx.layer_painter(layer).add(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(callback),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lights sit where AppKit's traffic lights do, so a header's
    /// left inset (84 points) clears all three.
    #[test]
    fn the_lights_fit_within_their_footprint() {
        let last = FIRST + 2.0 * SPACING + DIAMETER / 2.0;
        assert!(last <= WIDTH, "the zoom light must fit inside {WIDTH}");
        assert_eq!(WIDTH, fastframe_titlebar::TRAFFIC_LIGHTS_WIDTH);
    }

    /// The zoom light shows maximize while the window is restored and
    /// restore while it is maximized, each with its own label.
    #[test]
    fn the_zoom_glyph_follows_the_window_state() {
        let locale = crate::i18n::Locale::English;
        let (icon, label) = glyph(Light::Zoom, false, locale);
        assert_eq!(icon, Icon::TrafficMaximize);
        assert_eq!(label, "Maximize");
        let (icon, label) = glyph(Light::Zoom, true, locale);
        assert_eq!(icon, Icon::TrafficRestore);
        assert_eq!(label, "Restore");
        assert_eq!(glyph(Light::Close, false, locale).0, Icon::TrafficClose);
        assert_eq!(
            glyph(Light::Minimize, false, locale).0,
            Icon::TrafficMinimize
        );
    }

    /// The lights are a Linux-only replacement for the GNOME buttons, drawn
    /// only while the custom title bar is visible.
    #[test]
    fn the_lights_are_linux_only() {
        let ctx = egui::Context::default();
        // Off by default (no custom title bar chosen), on every platform.
        assert!(!active(&ctx));
    }
}
