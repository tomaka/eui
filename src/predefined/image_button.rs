use std::any::Any;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use Alignment;
use EventOutcome;
use Layout;
use Widget;

use predefined::Image;
use predefined::{MouseEnterEvent, MouseLeaveEvent};

/// An image that can be used as a button. Has a regular image and a hovered image.
pub struct ImageButton {
    hovered: AtomicBool,
    image_normal: Image,
    image_hovered: Image,
}

impl ImageButton {
    /// Initializes a new button.
    #[inline]
    pub fn new<S1, S2>(normal: S1, hovered: S2, height_per_width: f32) -> ImageButton
                       where S1: Into<String>, S2: Into<String>
    {
        ImageButton {
            hovered: AtomicBool::new(false),
            image_normal: Image::new(normal, height_per_width),
            image_hovered: Image::new(hovered, height_per_width),
        }
    }
}

impl Widget for ImageButton {
    #[inline]
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout {
        if self.hovered.load(Ordering::Relaxed) {
            self.image_hovered.build_layout(height_per_width, alignment)
        } else {
            self.image_normal.build_layout(height_per_width, alignment)
        }
    }

    #[inline]
    fn needs_rebuild(&self) -> bool {
        if self.hovered.load(Ordering::Relaxed) {
            self.image_hovered.needs_rebuild()
        } else {
            self.image_normal.needs_rebuild()
        }
    }

    #[inline]
    fn handle_event(&self, event: &Any, _: Option<usize>) -> EventOutcome {
        if let Some(event) = event.downcast_ref::<MouseEnterEvent>() {
            self.hovered.store(true, Ordering::Relaxed);

            EventOutcome {
                refresh_layout: true,
                propagate_to_parent: true,
            }

        } else if let Some(event) = event.downcast_ref::<MouseLeaveEvent>() {
            self.hovered.store(false, Ordering::Relaxed);

            EventOutcome {
                refresh_layout: true,
                propagate_to_parent: true,
            }

        } else {
            Default::default()
        }
    }
}
