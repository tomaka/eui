use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use Alignment;
use HorizontalAlignment;
use Layout;
use Matrix;
use Shape;
use VerticalAlignment;
use Widget;

pub struct Label {
    text: String,
    needs_refresh: AtomicBool,
}

impl Label {
    /// Initializes a new label.
    #[inline]
    pub fn new<S>(text: S) -> Label
                  where S: Into<String>
    {
        Label {
            text: text.into(),
            needs_refresh: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn set_text<S>(&mut self, text: S) where S: Into<String> {
        self.text = text.into();
        self.needs_refresh.store(true, Ordering::Relaxed);
    }
}

impl Widget for Label {
    #[inline]
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout {
        // TODO: everything here is temporary

        let text_ratio = 1.0 / self.text.len() as f32;       // TODO: wrong

        let matrix = if height_per_width > text_ratio {
            let y = match alignment.vertical {
                VerticalAlignment::Center => 0.0,
                VerticalAlignment::Top => 1.0 - text_ratio / height_per_width,
                VerticalAlignment::Bottom => -1.0 + text_ratio / height_per_width,
            };

            let scale = Matrix::scale_wh(1.0, text_ratio / height_per_width);
            let pos = Matrix::translate(0.0, y);
            pos * scale

        } else {
            let x = match alignment.horizontal {
                HorizontalAlignment::Center => 0.0,
                HorizontalAlignment::Left => -1.0 + height_per_width / text_ratio,
                HorizontalAlignment::Right => 1.0 - height_per_width / text_ratio,
            };

            let scale = Matrix::scale_wh(height_per_width / text_ratio, 1.0);
            let pos = Matrix::translate(x, 0.0);
            pos * scale
        };

        let shape = Shape::Text { matrix: matrix, text: self.text.clone() };
        Layout::Shapes(vec![shape])
    }

    #[inline]
    fn needs_rebuild(&self) -> bool {
        self.needs_refresh.swap(false, Ordering::Relaxed)
    }
}
