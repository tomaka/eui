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
pub use shape::Shape;
pub use ui::Ui;

pub mod predefined;

mod matrix;
mod shape;
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
    fn handle_event(&self, _event: &Any, _source_child: Option<usize>) -> EventOutcome {
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
    fn handle_event(&self, event: &Any, child: Option<usize>) -> EventOutcome {
        self.lock().unwrap().handle_event(event, child)
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
