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

/// Structure returned by `handle_event`, indicating information back to the library.
#[derive(Debug, Clone)]
pub struct EventOutcome {
    /// If `true`, the element's layout will be refreshed before the next draw. Default is `false`.
    pub refresh_layout: bool,
    /// If `true`, the event will be sent to the parent element. Default is `true`.
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

/// Implement this on types that contain the state of a widget.
pub trait Widget: Send + Sync + 'static {
    /// Returns a structure indicating the content of this widget.
    ///
    /// The `height_per_width` contains the ratio of the height of the widget divided by its width.
    /// The `alignment` is just an indication passed by the parent.
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout;

    /// This method is called before drawing. It should return `true` if the layout of this element
    /// should be rebuilt.
    ///
    /// The default implementation just returns `false`.
    ///
    /// This method is useful when dealing with animation. As long as the widget's content is
    /// moving, you should return `true`.
    #[inline]
    fn needs_rebuild(&self) -> bool {
        false
    }

    /// The widget received an event. It can update itself, then it should return an `EventOutcome`
    /// indicating the library what to do next. The default implementation returns
    /// `Default::default()`.
    ///
    /// The event can be:
    ///
    /// * `predefined::MouseEnterEvent`
    /// * `predefined::MouseLeaveEvent`
    /// * `predefined::MouseClick`
    /// * Any other event produced by another widget.
    ///
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

impl Default for HorizontalAlignment {
    fn default() -> HorizontalAlignment {
        HorizontalAlignment::Center
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerticalAlignment {
    Center,
    Top,
    Bottom,
}

impl Default for VerticalAlignment {
    fn default() -> VerticalAlignment {
        VerticalAlignment::Center
    }
}

pub enum Layout {
    AbsolutePositionned(Vec<(Matrix, Arc<Widget>)>),
    /// The content of the widget will be split in parts whose size depend on the weight of each
    /// child. Then the white spaces to the left and right of each child whose `collapse` value is
    /// `true` will be removed. The `alignment` is taken into account in order to align the
    /// elements once the children have been collapsed.
    HorizontalBar {
        /// How the children should be aligned once white spaces have been collapsed.
        alignment: HorizontalAlignment,
        /// List of children.
        children: Vec<Child>,
    },
    /// The same as `HorizontalBar`, but vertical.
    VerticalBar {
        /// How the children should be aligned once white spaces have been collapsed.
        alignment: VerticalAlignment,
        /// List of children.
        children: Vec<Child>,
    },
    Shapes(Vec<Shape>),
}

pub struct Child {
    pub child: Arc<Widget>,
    pub weight: i8,
    pub alignment: Alignment,
    pub collapse: bool,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
}
