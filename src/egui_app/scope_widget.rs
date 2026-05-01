//! Miniature scope widget rendered inside `DashboardScope` and `Scope` blocks.
//!
//! Each scope keeps a short history of live scalar samples and renders them as
//! a lightweight waveform preview. The same widget instance is reused for the
//! embedded scope and its popout window so both views stay in sync.

#![cfg(feature = "egui")]

use egui::{Align2, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use std::collections::VecDeque;

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

impl MiniScope {
    /// Create a new `MiniScope`.
    pub fn new(_id: impl std::hash::Hash) -> Self {
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

    /// Render the miniature scope into the given UI region.
    pub fn show(&mut self, ui: &mut Ui) {
        let available = ui.available_size_before_wrap();
        let desired = Vec2::new(available.x.max(40.0), available.y.max(30.0));
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
        self.paint(ui, rect);
    }

    fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        let frame_rect = rect.shrink(2.0);
        painter.rect_filled(frame_rect, 4.0, BG);
        painter.rect_stroke(
            frame_rect,
            4.0,
            Stroke::new(1.0, BORDER),
            egui::StrokeKind::Inside,
        );

        let has_title = !self.signal_name.is_empty() && frame_rect.height() >= 48.0;
        let title_height = if has_title { 18.0 } else { 0.0 };
        let plot_rect = Rect::from_min_max(
            Pos2::new(frame_rect.left() + 6.0, frame_rect.top() + 6.0 + title_height),
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
                [Pos2::new(plot_rect.left(), y), Pos2::new(plot_rect.right(), y)],
                Stroke::new(0.75, GRID),
            );
        }
        for i in 1..5 {
            let t = i as f32 / 5.0;
            let x = egui::lerp(plot_rect.left()..=plot_rect.right(), t);
            painter.line_segment(
                [Pos2::new(x, plot_rect.top()), Pos2::new(x, plot_rect.bottom())],
                Stroke::new(0.75, GRID),
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

        let (mut min_y, mut max_y) = self
            .samples
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min_y, max_y), (_, y)| {
                (min_y.min(*y), max_y.max(*y))
            });
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
            painter.line_segment([segment[0], segment[1]], Stroke::new(1.5, TRACE));
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
    let stroke = egui::Stroke::new(1.5, color);

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
