use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::symbols;
use tui::text::Span;
use tui::widgets::Block;
use tui::widgets::Widget;

#[derive(Debug, Clone)]
pub struct ThresholdGauge<'a> {
    block: Option<Block<'a>>,
    label: Span<'a>,
    use_unicode: bool,
    style: Style,
    gauge_style: Style,
    threshold: f64,
    max_value: f64,
    value: Option<f64>,
}

impl<'a> Default for ThresholdGauge<'a> {
    fn default() -> ThresholdGauge<'a> {
        ThresholdGauge {
            block: None,
            label: Span::raw(""),
            use_unicode: false,
            style: Style::default(),
            gauge_style: Style::default(),
            threshold: 100.0,
            max_value: 150.0,
            value: None,
        }
    }
}

impl<'a> ThresholdGauge<'a> {
    pub fn block(mut self, block: Block<'a>) -> ThresholdGauge<'a> {
        self.block = Some(block);
        self
    }

    pub fn value_and_max_value(mut self, value: f64, max_value: f64) -> ThresholdGauge<'a> {
        self.value = Some(value);
        self.max_value = max_value;
        self
    }

    pub fn threshold(mut self, threshold: f64) -> ThresholdGauge<'a> {
        self.threshold = threshold;
        self
    }

    pub fn label<T>(mut self, label: T) -> ThresholdGauge<'a>
    where
        T: Into<Span<'a>>,
    {
        self.label = label.into();
        self
    }

    pub fn style(mut self, style: Style) -> ThresholdGauge<'a> {
        self.style = style;
        self
    }

    pub fn gauge_style(mut self, style: Style) -> ThresholdGauge<'a> {
        self.gauge_style = style;
        self
    }

    pub fn use_unicode(mut self, unicode: bool) -> ThresholdGauge<'a> {
        self.use_unicode = unicode;
        self
    }
}

impl<'a> Widget for ThresholdGauge<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let gauge_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        buf.set_style(gauge_area, self.gauge_style);
        if gauge_area.height < 1 {
            return;
        }

        let ratio = match self.value {
            Some(v) => v / self.threshold,
            None => 0.0,
        };

        #[allow(unused_variables)]
        let ratio = ratio.min(self.max_value / self.threshold);
        let max_ratio = self.max_value / self.threshold;
        let threshold_ratio = self.threshold / self.max_value;

        let clamped_label_width = gauge_area.width.min(self.label.width() as u16);
        let label_col = 4;
        let label_row = gauge_area.top() + gauge_area.height / 2;

        // the gauge will be filled proportionally to the ratio
        let filled_width = f64::from(gauge_area.width)
            * (self.value.unwrap_or(0.0).min(self.max_value) / self.max_value);
        let end = if self.use_unicode {
            gauge_area.left() + filled_width.floor() as u16
        } else {
            gauge_area.left() + filled_width.round() as u16
        };

        for y in gauge_area.top()..gauge_area.bottom() {
            // render the filled area (left to end)
            for x in gauge_area.left()..end {
                let x_pos = x - gauge_area.left();
                let x_ratio = (f64::from(x_pos) / f64::from(gauge_area.width)) / threshold_ratio;

                // spaces are needed to apply the background styling
                buf.get_mut(x, y)
                    .set_symbol(" ")
                    .set_fg(self.gauge_style.bg.unwrap_or(Color::Reset))
                    .set_bg(get_gauge_color(x_ratio, threshold_ratio, max_ratio));
            }
            if self.use_unicode {
                let end_ratio = f64::from(end - 1) / f64::from(gauge_area.width) / threshold_ratio;
                buf.get_mut(end - 1, y)
                    .set_fg(self.gauge_style.bg.unwrap_or(Color::Reset))
                    .set_bg(get_gauge_color(end_ratio, threshold_ratio, max_ratio));
            }
        }

        let marker_pos: u16 =
            (threshold_ratio * f64::from(gauge_area.width)).floor() as u16 + gauge_area.left() + 2;
        for y in gauge_area.top()..gauge_area.bottom() {
            buf.get_mut(marker_pos, y)
                .set_symbol(" ")
                .set_fg(Color::White)
                .set_bg(Color::White);
        }

        // set the span
        buf.set_span(label_col, label_row, &self.label, clamped_label_width);
    }
}

fn get_unicode_block<'a>(frac: f64) -> &'a str {
    match (frac * 8.0).round() as u16 {
        1 => symbols::block::ONE_EIGHTH,
        2 => symbols::block::ONE_QUARTER,
        3 => symbols::block::THREE_EIGHTHS,
        4 => symbols::block::HALF,
        5 => symbols::block::FIVE_EIGHTHS,
        6 => symbols::block::THREE_QUARTERS,
        7 => symbols::block::SEVEN_EIGHTHS,
        8 => symbols::block::FULL,
        _ => " ",
    }
}

fn gradient_in(ratio: f64, threshold: f64, max_ratio: f64) -> u8 {
    let r = ratio.min(max_ratio);

    let c = (r / threshold).min(1.0) * 255.0;

    c as u8
}

fn gradient_out(ratio: f64, threshold: f64, max_ratio: f64) -> u8 {
    let r = ratio.min(max_ratio);
    let t = threshold * max_ratio;

    if r < t {
        return 255;
    }

    let c = 255.0 - ((r - t) / (max_ratio - t)).min(1.0) * 255.0;

    c.max(0.0).min(255.0) as u8
}

fn get_gauge_color<'a>(ratio: f64, threshold_ratio: f64, max_ratio: f64) -> Color {
    // let r: u8 = 0;
    let r: u8 = gradient_in(ratio, threshold_ratio, max_ratio);
    let g: u8 = gradient_out(ratio, threshold_ratio, max_ratio);
    let b: u8 = 0;

    Color::Rgb(r, g, b)
}
