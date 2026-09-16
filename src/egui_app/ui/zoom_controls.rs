use eframe::egui::{self, Pos2, Rect, Vec2};

/// Mutable view state read and updated by [`show_zoom_controls`].
pub struct ZoomViewState<'a> {
    /// Current zoom factor (1.0 = fit).
    pub zoom: &'a mut f32,
    /// Pan offset in screen pixels.
    pub pan: &'a mut Vec2,
    /// Set to `true` when the user requests a view reset.
    pub reset_requested: &'a mut bool,
    /// "Less colorful" rendering mode flag.
    pub monochrome: &'a mut bool,
}

/// Fixed canvas geometry used to keep the view centered while zooming.
#[derive(Clone, Copy)]
pub struct ZoomGeometry {
    /// Screen position of the controls overlay.
    pub fixed_pos: Pos2,
    /// Scale factor that fits the world into the view at zoom = 1.
    pub base_scale: f32,
    /// Bounding box of the world content.
    pub world_bounds: Rect,
    /// Screen-space origin of the world view (avail corner + margin).
    pub origin: Pos2,
    /// Screen-space point kept stationary while zooming.
    pub center: Pos2,
}

pub fn show_zoom_controls(
    ctx: &egui::Context,
    area_id: egui::Id,
    geom: ZoomGeometry,
    state: &mut ZoomViewState,
) {
    egui::Area::new(area_id)
        .fixed_pos(geom.fixed_pos)
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut zoom_by = |factor: f32| {
                        let old_zoom = *state.zoom;
                        let new_zoom = (old_zoom * factor).clamp(0.2, 30.0);
                        if (new_zoom - old_zoom).abs() <= f32::EPSILON {
                            return;
                        }

                        let s_old = geom.base_scale * old_zoom;
                        let s_new = geom.base_scale * new_zoom;
                        let world_x = (geom.center.x - geom.origin.x - state.pan.x) / s_old
                            + geom.world_bounds.left();
                        let world_y = (geom.center.y - geom.origin.y - state.pan.y) / s_old
                            + geom.world_bounds.top();
                        *state.zoom = new_zoom;
                        state.pan.x = geom.center.x
                            - ((world_x - geom.world_bounds.left()) * s_new + geom.origin.x);
                        state.pan.y = geom.center.y
                            - ((world_y - geom.world_bounds.top()) * s_new + geom.origin.y);
                    };

                    if ui.small_button("−").clicked() {
                        zoom_by(0.9);
                    }
                    if ui.small_button("+").clicked() {
                        zoom_by(1.1);
                    }
                    if ui.small_button("Reset").clicked() {
                        *state.reset_requested = true;
                    }

                    // The readout is expressed in the model's measurement unit:
                    // 100% == one screen pixel per model unit. `base_scale * zoom`
                    // is exactly that screen-pixels-per-model-unit scale, so the
                    // value is decoupled from the fit-to-view factor.
                    let percent = (geom.base_scale * *state.zoom * 100.0).round() as i32;
                    ui.label(format!("{}%", percent));

                    ui.separator();
                    ui.checkbox(state.monochrome, "Less color").on_hover_text(
                        "Flat Simulink-style blocks: white bodies with thin \
                         borders (areas keep their model colors)",
                    );
                    let mut dark = ui.visuals().dark_mode;
                    if ui
                        .checkbox(&mut dark, "Dark")
                        .on_hover_text("Toggle a dark canvas theme (signal lines stay visible)")
                        .changed()
                    {
                        ui.ctx().set_visuals(if dark {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        });
                    }
                });
            });
        });
}
