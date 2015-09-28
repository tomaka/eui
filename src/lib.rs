//! # Tutorial
//!
//! ## Step 1: build a hierarchy of objects representing the state of the UI.
//!
//! Create a structure of objects that represent the state of the user interface. Use `Arc`
//! throughout the hierarchy.
//!
//! ```ignore
//! let main_menu = MainMenuUiState {
//!     title: Arc::new(TextLabelUi { text: "My awesome game!" }),
//!     buttons: vec![
//!         Arc::new(ButtonState { title: "Start" }),
//!         Arc::new(ButtonState { title: "Options" }),
//!         Arc::new(ButtonState { title: "Quit" }),
//!     ]
//! }
//! ```
//!
//! ## Step 2: implement the `Widget` trait on your ui elements.
//!
//! This trait implementation tells `eui` what the layout of your user interface is.
//!
//! ```ignore
//! impl eui::Widget for MainMenuUiState {
//!     fn build_layout(&self) -> eui::Layout {
//!         let mut list = vec![self.title.clone()];
//!         for b in &self.buttons { list.push(b.clone()); }
//!         eui::Layout::Vertical(list)
//!     }
//! }
//! ```
//!
//! ## Step 3: create a `eui::Ui` object and pass your state to it.
//!
//! The `eui` library will build a secondary hierarchy that contains the positions, rotation and
//! scale of each component. This hierarchy is handled internally.
//!
//! At any moment you can call the `draw` method on your `Ui` to obtain a list of things to draw
//! when you render your user interface.
//!
//! You can still access and/or modify your state through the `widget()` method of the `Ui` object.
//!
//! ## Step 4: send events to the `eui::Ui`
//!
//! Whenever the mouse cursor is moved, a key is pressed, etc. you should call the appropriate
//! method on the `Ui` object to update it.
//!
//! These calls will be propagated to the methods of the `Widget` trait so that the state can
//! update itself.
//!
extern crate time;

use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;

pub use matrix::Matrix;
pub use ui::Ui;

pub mod predefined;

mod matrix;
mod ui;

pub struct EventOutcome {
    pub refresh_layout: bool,
    pub propagate_to_parent: bool,
}

impl Default for EventOutcome {
    fn default() -> EventOutcome {
        EventOutcome {
            refresh_layout: false,
            propagate_to_parent: true,
        }
    }
}

pub trait Widget: Send + Sync + 'static {
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout;

    #[inline]
    fn needs_rebuild(&self) -> bool {
        false
    }

    #[inline]
    fn handle_event(&self, _event: Box<Any>) -> EventOutcome {
        Default::default()
    }
}

impl<T> Widget for Mutex<T> where T: Widget {
    #[inline]
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout {
        self.lock().unwrap().build_layout(height_per_width, alignment)
    }

    #[inline]
    fn needs_rebuild(&self) -> bool {
        self.lock().unwrap().needs_rebuild()
    }

    #[inline]
    fn handle_event(&self, event: Box<Any>) -> EventOutcome {
        self.lock().unwrap().handle_event(event)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Center,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerticalAlignment {
    Center,
    Top,
    Bottom,
}

pub enum Layout {
    AbsolutePositionned(Vec<(Matrix, Arc<Widget>)>),
    HorizontalBar {
        alignment: VerticalAlignment,
        children: Vec<(i8, Arc<Widget>)>,
    },
    VerticalBar,
    Shapes(Vec<Shape>),
}

/// A shape that can be drawn by any of the UI's components.
#[derive(Clone, Debug)]
pub enum Shape {
    Text {
        matrix: Matrix,
        text: String,
    },
    Image {
        matrix: Matrix,
        name: String,
    },
}

impl Shape {
    #[inline]
    pub fn apply_matrix(self, outer: &Matrix) -> Shape {
        match self {
            Shape::Text { matrix, text } => Shape::Text { matrix: *outer * matrix, text: text },
            Shape::Image { matrix, name } => Shape::Image { matrix: *outer * matrix, name: name },
        }
    }

    /// Returns true if the point's coordinates hit the shape.
    pub fn hit_test(&self, point: &[f32; 2]) -> bool {
        /// Calculates whether the point is in a rectangle multiplied by a matrix.
        fn test(matrix: &Matrix, point: &[f32; 2]) -> bool {
            // We start by calculating the positions of the four corners of the shape in viewport
            // coordinates, so that they can be compared with the point which is already in
            // viewport coordinates.

            let top_left = *matrix * [-1.0, 1.0, 1.0];
            let top_left = [top_left[0] / top_left[2], top_left[1] / top_left[2]];

            let top_right = *matrix * [1.0, 1.0, 1.0];
            let top_right = [top_right[0] / top_right[2], top_right[1] / top_right[2]];

            let bot_left = *matrix * [-1.0, -1.0, 1.0];
            let bot_left = [bot_left[0] / bot_left[2], bot_left[1] / bot_left[2]];

            let bot_right = *matrix * [1.0, -1.0, 1.0];
            let bot_right = [bot_right[0] / bot_right[2], bot_right[1] / bot_right[2]];

            // The point is within our rectangle if and only if it is on the right side of each
            // border of the rectangle (taken in the right order).
            //
            // To check this, we calculate the dot product of the vector `point - corner` with
            // `next_corner - corner`. If the value is positive, then the angle is inferior to
            // 90°. If the the value is negative, the angle is superior to 90° and we know that
            // the cursor is outside of the rectangle.

            if (point[0] - top_left[0]) * (top_right[0] - top_left[0]) +
               (point[1] - top_left[1]) * (top_right[1] - top_left[1]) < 0.0
            {
                return false;
            }

            if (point[0] - top_right[0]) * (bot_right[0] - top_right[0]) +
               (point[1] - top_right[1]) * (bot_right[1] - top_right[1]) < 0.0
            {
                return false;
            }

            if (point[0] - bot_right[0]) * (bot_left[0] - bot_right[0]) +
               (point[1] - bot_right[1]) * (bot_left[1] - bot_right[1]) < 0.0
            {
                return false;
            }

            if (point[0] - bot_left[0]) * (top_left[0] - bot_left[0]) +
               (point[1] - bot_left[1]) * (top_left[1] - bot_left[1]) < 0.0
            {
                return false;
            }

            true
        }

        match self {
            &Shape::Text { ref matrix, .. } => {
                test(matrix, point)
            },

            &Shape::Image { ref matrix, .. } => {
                test(matrix, point)
            },
        }
    }
}
