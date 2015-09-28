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
use std::mem;

pub use matrix::Matrix;

pub mod predefined;

mod matrix;

pub trait Widget: Send + Sync + 'static {
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout;

    #[inline]
    fn needs_rebuild(&self) -> bool {
        false
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

struct Node {
    /// Absolute matrix (relative to root)
    matrix: Matrix,
    state: Arc<Widget>,
    children: Vec<Node>,
    shapes: Vec<Shape>,
}

impl Node {
    #[inline]
    fn needs_rebuild(&self) -> bool {
        if self.state.needs_rebuild() {
            return true;
        }

        for child in &self.children {
            if child.needs_rebuild() {
                return true;
            }
        }

        false
    }

    fn rebuild_children(&mut self, parent_height_per_width: f32, alignment: Alignment) {
        let my_height_per_width = self.matrix.0[1][1] / self.matrix.0[0][0];

        let mut state_children = self.state.build_layout(my_height_per_width, alignment);

        self.shapes = match state_children {
            Layout::Shapes(ref mut look) => mem::replace(look, Vec::new()),
            _ => Vec::new()
        };

        self.children = match state_children {
            Layout::AbsolutePositionned(list) => {
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: VerticalAlignment::Center,
                };

                list.into_iter().map(|(matrix, w)| {
                    let mut node = Node {
                        matrix: self.matrix.clone() * matrix,
                        state: w, 
                        children: Vec::new(),
                        shapes: Vec::new(),
                    };

                    node.rebuild_children(my_height_per_width, children_alignment);
                    node
                }).collect()
            },

            Layout::HorizontalBar { alignment, children } => {
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: alignment,
                };

                let elems_len = 1.0 / children.iter().fold(0, |a, b| a + b.0) as f32;
                let scale = Matrix::scale_wh(elems_len, 1.0);

                let mut offset = 0;
                children.into_iter().map(|(weight, widget)| {
                    let position = (offset as f32 * 2.0 + 1.0) * elems_len - 1.0;
                    let position = Matrix::translate(position, 0.0);

                    offset += weight;

                    let mut node = Node {
                        matrix: self.matrix * position * scale,
                        state: widget,
                        children: Vec::new(),
                        shapes: Vec::new(),
                    };

                    node.rebuild_children(my_height_per_width, children_alignment);
                    node
                }).collect()
            },

            Layout::Shapes(_) => Vec::new(),

            _ => unimplemented!()
        };
    }

    fn build_shapes(&self) -> Vec<Shape> {
        let mut result = Vec::new();

        for c in &self.children {
            for s in c.build_shapes() { result.push(s); }
        }

        for s in &self.shapes {
            result.push(s.clone().apply_matrix(&self.matrix));
        }

        result
    }
}

/// Main struct of this library. Handles the UI as a whole.
pub struct Ui<S> {
    viewport_height_per_width: f32,
    widget: Arc<S>,
    main_node: Mutex<Node>,
}

impl<S> Ui<S> where S: Widget {
    /// Builds a new `Ui`.
    pub fn new(state: S, viewport_height_per_width: f32) -> Ui<S> {
        let state = Arc::new(state);

        let mut main_node = Node {
            matrix: Matrix::identity(),
            state: state.clone() as Arc<_>,
            children: Vec::new(),
            shapes: Vec::new(),
        };

        main_node.rebuild_children(viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        Ui {
            viewport_height_per_width: viewport_height_per_width,
            widget: state,
            main_node: Mutex::new(main_node),
        }
    }

    /// Rebuilds the UI after the state has been changed.
    #[inline]
    pub fn rebuild(&self) {
        self.main_node.lock().unwrap().rebuild_children(self.viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });
    }

    /// "Draws" the UI by returning a list of shapes. The list is ordered from bottom to top (in
    /// other words, shapes at the start of the list can be obstructed by shapes further ahead
    /// in the list).
    #[inline]
    pub fn draw(&self) -> Vec<Shape> {
        let mut main_node = self.main_node.lock().unwrap();

        if main_node.needs_rebuild() {
            main_node.rebuild_children(self.viewport_height_per_width, Alignment {
                horizontal: HorizontalAlignment::Center,
                vertical: VerticalAlignment::Center,
            });
        }

        main_node.build_shapes()
    }

    /// Changes the height per width ratio of the viewport and rebuilds the UI.
    #[inline]
    pub fn set_viewport_height_per_width(&mut self, value: f32) {
        if self.viewport_height_per_width != value {
            self.viewport_height_per_width = value;
            self.rebuild();
        }
    }

    pub fn set_cursor(&mut self, cursor: Option<[f32; 2]>) {
        unimplemented!()
    }

    pub fn set_cursor_down(&mut self, down: bool) {
        unimplemented!()
    }

    /// Returns a reference to the main widget stored in the object.
    #[inline]
    pub fn widget(&self) -> &S {
        &self.widget
    }
}





/// Trait describing an event.
pub trait Event: Any {}
impl<T> Event for T where T: Any {}

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
    pub fn apply_matrix(self, outer: &Matrix) -> Shape {
        match self {
            Shape::Text { matrix, text } => Shape::Text { matrix: *outer * matrix, text: text },
            Shape::Image { matrix, name } => Shape::Image { matrix: *outer * matrix, name: name },
        }
    }
}
