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
use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;

pub use matrix::Matrix;

pub mod predefined;

mod matrix;

pub trait Widget: Send + Sync + 'static {
    fn build_layout(&self) -> Layout;
}

impl<T> Widget for Mutex<T> where T: Widget {
    fn build_layout(&self) -> Layout {
        self.lock().unwrap().build_layout()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alignment {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum Layout {
    AbsolutePositionned(Vec<Arc<Widget>>),
    HorizontalBar {
        alignment: Alignment,
        children: Vec<(i8, Arc<Widget>)>,
    },
    VerticalBar,
    Shapes(Box<WidgetLook>),
}

pub trait WidgetLook {
    fn draw(&self) -> Vec<Shape>;
}

impl WidgetLook for Shape {
    fn draw(&self) -> Vec<Shape> {
        vec![self.clone()]
    }
}

impl WidgetLook for Vec<Shape> {
    fn draw(&self) -> Vec<Shape> {
        self.clone()
    }
}

impl WidgetLook for () {
    fn draw(&self) -> Vec<Shape> {
        vec![]
    }
}

struct Node {
    matrix: Matrix,
    state: Arc<Widget>,
    children: Vec<Node>,
    shapes: Vec<Shape>,
}

impl Node {
    fn rebuild_children(&mut self, parent_height_per_width: f32) {
        let my_height_per_width = parent_height_per_width * self.matrix.0[1][1]
                                                                            / self.matrix.0[0][0];

        let state_children = self.state.build_layout();

        self.shapes = match state_children {
            Layout::Shapes(ref look) => {
                look.draw()
            },

            _ => Vec::new()
        };

        self.children = match state_children {
            Layout::AbsolutePositionned(list) => {
                list.into_iter().map(|w| {
                    Node {
                        matrix: self.matrix.clone(),
                        state: w, 
                        children: Vec::new(),
                        shapes: Vec::new(),
                    }
                }).collect()
            },

            Layout::HorizontalBar { alignment, children } => {
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

                    node.rebuild_children(my_height_per_width);
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
            for s in c.build_shapes() { result.push(s.apply_matrix(&self.matrix)); }
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
    main_node: Node,
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

        main_node.rebuild_children(viewport_height_per_width);

        Ui {
            viewport_height_per_width: viewport_height_per_width,
            widget: state,
            main_node: main_node,
        }
    }

    /// Rebuilds the UI after the state has been changed.
    pub fn rebuild(&mut self) {
        self.main_node.rebuild_children(self.viewport_height_per_width);
    }

    /// "Draws" the UI by returning a list of shapes. The list is ordered from bottom to top (in
    /// other words, shapes at the start of the list can be obstructed by shapes further ahead
    /// in the list).
    pub fn draw(&self) -> Vec<Shape> {
        self.main_node.build_shapes()
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
