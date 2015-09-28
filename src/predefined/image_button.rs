use Layout;
use Event;
use Matrix;
use Shape;
use Widget;

use predefined::Image;
use predefined::{MouseEnterEvent, MouseLeaveEvent};

pub struct ImageButton {
    hovered: bool,
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
            hovered: false,
            image_normal: Image::new(normal, height_per_width),
            image_hovered: Image::new(hovered, height_per_width),
        }
    }
}

impl Widget for ImageButton {
    #[inline]
    fn build_layout(&self, height_per_width: f32) -> Layout {
        if self.hovered {
            self.image_hovered.build_layout(height_per_width)
        } else {
            self.image_normal.build_layout(height_per_width)
        }
    }
}

/*impl Widget for Button {
    #[inline]
    fn draw(&self) -> Vec<Shape> {
        if self.hovered {
            self.image_hovered.draw()
        } else {
            self.image_normal.draw()
        }
    }

    #[inline]
    fn set_dimensions(&mut self, matrix: &Matrix, viewport_height_per_width: f32)
                      -> Vec<Box<Event>>
    {
        // TODO: propagate events

        self.image_normal.set_dimensions(matrix, viewport_height_per_width);
        self.image_hovered.set_dimensions(matrix, viewport_height_per_width);

        vec![]
    }

    #[inline]
    fn set_cursor(&mut self, cursor: Option<[f32; 2]>) -> Vec<Box<Event>> {
        let hovered = match cursor {
            Some(pos) => pos[0] >= -1.0 && pos[0] <= 1.0 && pos[1] >= -1.0 && pos[1] <= 1.0,
            None => false,
        };

        let events = match (self.hovered, hovered) {
            (false, true) => vec![Box::new(MouseEnterEvent) as Box<Event>],
            (true, false) => vec![Box::new(MouseLeaveEvent) as Box<Event>],
            _ => vec![]
        };

        self.hovered = hovered;

        events
    }
}*/
