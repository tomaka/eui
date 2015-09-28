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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::mem;
use std::ops::Deref;

pub use matrix::Matrix;

pub mod predefined;

mod matrix;

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
    fn handle_event(&self, event: Box<Any>) -> EventOutcome {
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

struct Node {
    /// Absolute matrix (relative to root)
    matrix: Matrix,
    state: Arc<Widget>,
    children: Vec<Node>,
    shapes: Vec<Shape>,
    needs_rebuild: bool,
}

impl Node {
    #[inline]
    fn needs_rebuild(&mut self) -> bool {
        if self.needs_rebuild {
            self.needs_rebuild = false;
            return true;
        }

        if self.state.needs_rebuild() {
            return true;
        }

        for child in &mut self.children {
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
            Layout::Shapes(ref mut look) => {
                let shapes = mem::replace(look, Vec::new());
                shapes.into_iter().map(|s| s.apply_matrix(&self.matrix)).collect()
            },
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
                        needs_rebuild: false,
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
                        needs_rebuild: false,
                    };

                    node.rebuild_children(my_height_per_width, children_alignment);
                    node
                }).collect()
            },

            Layout::Shapes(_) => Vec::new(),

            _ => unimplemented!()
        };

        self.needs_rebuild = false;
    }

    fn build_shapes(&self) -> Vec<Shape> {
        let mut result = Vec::new();

        for c in &self.children {
            for s in c.build_shapes() { result.push(s); }
        }

        for s in &self.shapes {
            result.push(s.clone());
        }

        result
    }

    fn send_event(&self, event: Box<Any>) -> EventOutcome {
        self.state.handle_event(event)
    }

    fn mouse_update(&mut self, mouse: Option<[f32; 2]>) -> EventOutcome {
        for child in &mut self.children {
            let outcome = child.mouse_update(mouse);

            if outcome.refresh_layout {
                child.needs_rebuild = true;
            }

            if outcome.propagate_to_parent {
                // TODO: implement
            }

            // TODO: break if event handled
        }

        let hit = if let Some(mouse) = mouse {
            self.shapes.iter().find(|s| s.hit_test(&mouse)).is_some()
        } else {
            false
        };

        // TODO: do not send these events twice
        if hit {
            self.send_event(Box::new(predefined::MouseEnterEvent))
        } else {
            self.send_event(Box::new(predefined::MouseLeaveEvent))
        }
    }
}

/// Main struct of this library. Handles the UI as a whole.
pub struct Ui<S> {
    viewport_height_per_width: f32,
    widget: Arc<S>,
    main_node: Mutex<Node>,
    hovering: AtomicBool,
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
            needs_rebuild: false,
        };

        main_node.rebuild_children(viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        Ui {
            viewport_height_per_width: viewport_height_per_width,
            widget: state,
            main_node: Mutex::new(main_node),
            hovering: AtomicBool::new(false),
        }
    }

    /// Rebuilds the UI after the state has been changed.
    #[inline]
    pub fn rebuild(&self) {
        self.main_node.lock().unwrap().rebuild_children(self.viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        // TODO: update mouse?
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
    // TODO: use &self not &mut self
    #[inline]
    pub fn set_viewport_height_per_width(&mut self, value: f32) {
        if self.viewport_height_per_width != value {
            self.viewport_height_per_width = value;
            self.rebuild();
        }
    }

    pub fn set_cursor(&self, cursor: Option<[f32; 2]>) {
        let mut main_node = self.main_node.lock().unwrap();

        let outcome = main_node.mouse_update(cursor);
        if outcome.refresh_layout {
            main_node.needs_rebuild = true;
        }

        // FIXME: update "hovering"
    }

    pub fn set_cursor_down(&mut self, down: bool) {
        unimplemented!()
    }

    /// Returns true if the mouse is hovering one of the elements of the UI.
    pub fn is_hovering(&self) -> bool {
        self.hovering.load(Ordering::Relaxed)
    }

    /// Returns a reference to the main widget stored in the object.
    #[inline]
    pub fn widget(&self) -> &S {
        &self.widget
    }
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
