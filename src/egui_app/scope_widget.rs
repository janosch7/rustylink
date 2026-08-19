//! Miniature scope widget rendered inside `DashboardScope` and `Scope` blocks.
//!
//! Embedded scopes stay lightweight and `Send` by storing only sample history.
//! Popout rendering keeps a persistent `liveplot` panel per scope so controls,
//! legend, and zoom state remain stable across frames.

#![cfg(feature = "egui")]

use egui::{Align2, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use liveplot::data::scope::{AxisType, ScopeType, ValueFormat};
use liveplot::{LivePlotPanel, PlotPoint, PlotSink, Trace, channel_plot};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

const MAX_SAMPLES: usize = 512;
const BG: Color32 = Color32::from_rgb(18, 20, 24);
const GRID: Color32 = Color32::from_rgb(52, 60, 72);
const BORDER: Color32 = Color32::from_rgb(88, 96, 108);
const TRACE: Color32 = Color32::from_rgb(72, 214, 120);
const TEXT: Color32 = Color32::from_rgb(210, 220, 228);

/// State for a single miniature scope instance.
pub struct MiniScope {
    signal_name: String,
    next_x: f64,
    samples: VecDeque<(f64, f64)>,
}

struct ScopePopoutState {
    sink: PlotSink,
    panel: LivePlotPanel,
    trace: Trace,
    last_sent_x: Option<f64>,
    trace_label: String,
}

thread_local! {
    static SCOPE_POPOUT_STATE: RefCell<HashMap<(u64, String), ScopePopoutState>> =
        RefCell::new(HashMap::new());
    static SCOPE_EMBEDDED_STATE: RefCell<HashMap<(u64, String), ScopePopoutState>> =
        RefCell::new(HashMap::new());
}

fn configure_popout_panel(panel: &mut LivePlotPanel, signal_name: &str) {
    configure_panel(panel, signal_name, None, false);
}

fn configure_embedded_panel(panel: &mut LivePlotPanel, signal_name: &str, size: Vec2) {
    configure_panel(panel, signal_name, Some(size), true);
}

fn configure_panel(
    panel: &mut LivePlotPanel,
    signal_name: &str,
    size: Option<Vec2>,
    compact: bool,
) {
    panel.traces_data.max_points = MAX_SAMPLES;
    panel.min_height_for_top_bar = if compact { f32::INFINITY } else { 220.0 };
    panel.min_width_for_sidebar = if compact { f32::INFINITY } else { 520.0 };
    panel.min_height_for_sidebar = if compact { f32::INFINITY } else { 260.0 };
    panel.compact = compact;
    if compact {
        panel.liveplot_panel.set_tick_label_thresholds(190.0, 130.0);
        panel
            .liveplot_panel
            .set_legend_thresholds(f32::INFINITY, f32::INFINITY);
    } else {
        panel.liveplot_panel.set_tick_label_thresholds(360.0, 220.0);
        panel.liveplot_panel.set_legend_thresholds(520.0, 260.0);
    }

    for scope in panel.liveplot_panel.get_data_mut() {
        scope.scope_type = ScopeType::XYScope;
        scope.show_legend = !compact;
        scope.show_info_in_legend = false;
        scope.force_hide_legend = compact;
        scope.x_axis.axis_type = AxisType::Value(ValueFormat::default());
        let show_axis_names = size
            .map(|size| size.x >= 250.0 && size.y >= 150.0)
            .unwrap_or(true);
        scope.x_axis.name = if compact && !show_axis_names {
            None
        } else {
            Some("Sample".to_string())
        };
        scope.x_axis.auto_fit = true;
        scope.y_axis.name = if compact && !show_axis_names {
            None
        } else {
            Some(if signal_name.trim().is_empty() {
                "Value".to_string()
            } else {
                signal_name.to_string()
            })
        };
        scope.y_axis.auto_fit = true;
        scope.y_axis.log_scale = false;
        scope.pause_on_click = false;
    }
}

pub fn clear_popout_state(viewer_instance_id: u64, scope_key: &str) {
    SCOPE_POPOUT_STATE.with(|state| {
        state
            .borrow_mut()
            .remove(&(viewer_instance_id, scope_key.to_string()));
    });
}

impl MiniScope {
    /// Create a new `MiniScope`.
    pub fn new(_id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            signal_name: String::new(),
            next_x: 0.0,
            samples: VecDeque::with_capacity(MAX_SAMPLES),
        }
    }

    pub fn set_signal_name(&mut self, signal_name: impl Into<String>) {
        let signal_name = signal_name.into();
        if !signal_name.trim().is_empty() {
            self.signal_name = signal_name;
        }
    }

    pub fn push_sample(&mut self, value: f64) {
        if !value.is_finite() {
            return;
        }
        self.samples.push_back((self.next_x, value));
        self.next_x += 1.0;
        while self.samples.len() > MAX_SAMPLES {
            self.samples.pop_front();
        }
    }

    pub fn show_embedded(&mut self, ui: &mut Ui, viewer_instance_id: u64, scope_key: &str) {
        let available = ui.available_size_before_wrap();
        let desired = Vec2::new(available.x.max(40.0), available.y.max(30.0));
        if self.samples.is_empty() || desired.x < 40.0 || desired.y < 30.0 {
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
            self.paint_compact(ui, rect);
            return;
        }

        let state_key = (viewer_instance_id, scope_key.to_string());
        SCOPE_EMBEDDED_STATE.with(|state| {
            let mut cache = state.borrow_mut();
            let embedded = cache.entry(state_key).or_insert_with(|| {
                let (sink, rx) = channel_plot();
                let mut panel = LivePlotPanel::new(rx);
                let trace_label = if self.signal_name.trim().is_empty() {
                    "signal".to_string()
                } else {
                    self.signal_name.clone()
                };
                configure_embedded_panel(&mut panel, &self.signal_name, desired);
                let trace = sink.create_trace(trace_label.clone(), None::<String>);
                ScopePopoutState {
                    sink,
                    panel,
                    trace,
                    last_sent_x: None,
                    trace_label,
                }
            });

            configure_embedded_panel(&mut embedded.panel, &self.signal_name, desired);
            if !self.signal_name.trim().is_empty() && embedded.trace_label != self.signal_name {
                embedded.trace_label = self.signal_name.clone();
                embedded
                    .sink
                    .set_trace_info(&embedded.trace, self.signal_name.clone());
            }

            let new_points: Vec<PlotPoint> = self
                .samples
                .iter()
                .filter(|(x, _)| embedded.last_sent_x.map(|last| *x > last).unwrap_or(true))
                .map(|(x, y)| PlotPoint { x: *x, y: *y })
                .collect();
            if !new_points.is_empty() {
                let _ = embedded.sink.send_points(&embedded.trace, new_points);
                embedded.last_sent_x = self.samples.back().map(|(x, _)| *x);
            }

            embedded.panel.update_embedded(ui);
        });
    }

    pub fn show_popout(&mut self, ui: &mut Ui, viewer_instance_id: u64, scope_key: &str) {
        if self.samples.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No live data");
            });
            return;
        }

        let state_key = (viewer_instance_id, scope_key.to_string());
        SCOPE_POPOUT_STATE.with(|state| {
            let mut cache = state.borrow_mut();
            let popout = cache.entry(state_key).or_insert_with(|| {
                let (sink, rx) = channel_plot();
                let mut panel = LivePlotPanel::new(rx);
                let trace_label = if self.signal_name.trim().is_empty() {
                    "signal".to_string()
                } else {
                    self.signal_name.clone()
                };
                configure_popout_panel(&mut panel, &self.signal_name);
                let trace = sink.create_trace(trace_label.clone(), None::<String>);
                ScopePopoutState {
                    sink,
                    panel,
                    trace,
                    last_sent_x: None,
                    trace_label,
                }
            });

            if !self.signal_name.trim().is_empty() {
                configure_popout_panel(&mut popout.panel, &self.signal_name);
                if popout.trace_label != self.signal_name {
                    popout.trace_label = self.signal_name.clone();
                    popout
                        .sink
                        .set_trace_info(&popout.trace, self.signal_name.clone());
                }
            }

            let new_points: Vec<PlotPoint> = self
                .samples
                .iter()
                .filter(|(x, _)| popout.last_sent_x.map(|last| *x > last).unwrap_or(true))
                .map(|(x, y)| PlotPoint { x: *x, y: *y })
                .collect();
            if !new_points.is_empty() {
                let _ = popout.sink.send_points(&popout.trace, new_points);
                popout.last_sent_x = self.samples.back().map(|(x, _)| *x);
            }

            popout.panel.update_embedded(ui);
        });
    }

    fn paint_compact(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        let frame_rect = rect.shrink(2.0);
        painter.rect_filled(frame_rect, 4.0, BG);
        painter.rect_stroke(
            frame_rect,
            4.0,
            Stroke::new(1.0_f32, BORDER),
            egui::StrokeKind::Inside,
        );

        let has_title = !self.signal_name.is_empty() && frame_rect.height() >= 48.0;
        let title_height = if has_title { 18.0 } else { 0.0 };
        let plot_rect = Rect::from_min_max(
            Pos2::new(
                frame_rect.left() + 6.0,
                frame_rect.top() + 6.0 + title_height,
            ),
            Pos2::new(frame_rect.right() - 6.0, frame_rect.bottom() - 6.0),
        );

        if plot_rect.width() <= 4.0 || plot_rect.height() <= 4.0 {
            return;
        }

        if has_title {
            painter.text(
                Pos2::new(frame_rect.left() + 8.0, frame_rect.top() + 7.0),
                Align2::LEFT_TOP,
                &self.signal_name,
                egui::FontId::proportional(11.0),
                TEXT,
            );
        }

        for i in 1..4 {
            let t = i as f32 / 4.0;
            let y = egui::lerp(plot_rect.top()..=plot_rect.bottom(), t);
            painter.line_segment(
                [
                    Pos2::new(plot_rect.left(), y),
                    Pos2::new(plot_rect.right(), y),
                ],
                Stroke::new(0.75_f32, GRID),
            );
        }
        for i in 1..5 {
            let t = i as f32 / 5.0;
            let x = egui::lerp(plot_rect.left()..=plot_rect.right(), t);
            painter.line_segment(
                [
                    Pos2::new(x, plot_rect.top()),
                    Pos2::new(x, plot_rect.bottom()),
                ],
                Stroke::new(0.75_f32, GRID),
            );
        }

        let Some((first_x, _)) = self.samples.front().copied() else {
            painter.text(
                plot_rect.center(),
                Align2::CENTER_CENTER,
                "No live data",
                egui::FontId::proportional(11.0),
                TEXT,
            );
            return;
        };
        let (last_x, _) = self.samples.back().copied().unwrap_or((first_x + 1.0, 0.0));

        let (mut min_y, mut max_y) = self.samples.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(min_y, max_y), (_, y)| (min_y.min(*y), max_y.max(*y)),
        );
        if !min_y.is_finite() || !max_y.is_finite() {
            min_y = -1.0;
            max_y = 1.0;
        }
        if (max_y - min_y).abs() < f64::EPSILON {
            min_y -= 1.0;
            max_y += 1.0;
        }
        let x_span = (last_x - first_x).max(1.0);
        let y_span = (max_y - min_y).max(f64::EPSILON);

        let mut points = Vec::with_capacity(self.samples.len());
        for (x, y) in &self.samples {
            let tx = ((*x - first_x) / x_span) as f32;
            let ty = ((*y - min_y) / y_span) as f32;
            points.push(Pos2::new(
                egui::lerp(plot_rect.left()..=plot_rect.right(), tx),
                egui::lerp(plot_rect.bottom()..=plot_rect.top(), ty),
            ));
        }
        for segment in points.windows(2) {
            painter.line_segment([segment[0], segment[1]], Stroke::new(1.5_f32, TRACE));
        }
    }
}

/// Draw a simple waveform glyph (using raw painter strokes) inside the given
/// rectangle.  This is a lightweight fallback that does not depend on the full
/// [`liveplot`] panel infrastructure.
pub fn draw_scope_glyph(ui: &mut Ui, rect: Rect) {
    let inner = rect.shrink(6.0);
    if inner.width() < 10.0 || inner.height() < 10.0 {
        return;
    }

    let painter = ui.painter();
    let color = Color32::from_rgb(50, 200, 50);
    let stroke = egui::Stroke::new(1.5_f32, color);

    // Draw a stylized sine wave
    let n = 60;
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let x = inner.left() + t * inner.width();
        let y =
            inner.center().y - (t * 2.0 * std::f32::consts::PI * 2.0).sin() * inner.height() * 0.35;
        points.push(egui::pos2(x, y));
    }

    // Draw background
    painter.rect_filled(inner, 2.0, Color32::from_rgb(30, 30, 30));

    // Draw the waveform line
    for w in points.windows(2) {
        painter.line_segment([w[0], w[1]], stroke);
    }
}
