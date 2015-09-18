use std::any::Any;
use std::ops;

pub mod layout;
pub mod predefined;

/// A 3x3 matrix. The data is stored in column-major.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix(pub [[f32; 3]; 3]);

impl Matrix {
    /// Builds an identity matrix, in other words a matrix that has no effect.
    #[inline]
    pub fn identity() -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Builds a matrix that will rescale both width and height of a given factor.
    #[inline]
    pub fn scale(factor: f32) -> Matrix {
        Matrix([
            [factor,   0.0 , 0.0],
            [  0.0 , factor, 0.0],
            [  0.0 ,   0.0 , 1.0],
        ])
    }

    /// Builds a matrix that will multiply the width and height by a certain factor.
    #[inline]
    pub fn scale_wh(w: f32, h: f32) -> Matrix {
        Matrix([
            [ w,  0.0, 0.0],
            [0.0,  h,  0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Builds a matrix that will translate the object.
    #[inline]
    pub fn translate(x: f32, y: f32) -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [ x,   y,  1.0],
        ])
    }
}

impl ops::Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let me = self.0;
        let other = other.0;

        let a = me[0][0] * other[0][0] + me[1][0] * other[0][1] + me[2][0] * other[0][2];
        let b = me[0][0] * other[1][0] + me[1][0] * other[1][1] + me[2][0] * other[1][2];
        let c = me[0][0] * other[2][0] + me[1][0] * other[2][1] + me[2][0] * other[2][2];
        let d = me[0][1] * other[0][0] + me[1][1] * other[0][1] + me[2][1] * other[0][2];
        let e = me[0][1] * other[1][0] + me[1][1] * other[1][1] + me[2][1] * other[1][2];
        let f = me[0][1] * other[2][0] + me[1][1] * other[2][1] + me[2][1] * other[2][2];
        let g = me[0][2] * other[0][0] + me[1][2] * other[0][1] + me[2][2] * other[0][2];
        let h = me[0][2] * other[1][0] + me[1][2] * other[1][1] + me[2][2] * other[1][2];
        let i = me[0][2] * other[2][0] + me[1][2] * other[2][1] + me[2][2] * other[2][2];

        Matrix([
            [a, b, c],
            [d, e, f],
            [g, h, i],
        ])
    }
}

impl From<[[f32; 3]; 3]> for Matrix {
    fn from(val: [[f32; 3]; 3]) -> Matrix {
        Matrix(val)
    }
}

/// Represents a widget of the UI.
pub trait Widget {
    /// Returns the list of shapes that must be drawn to display this widget. The list must be
    /// ordered from bottom to top.
    ///
    /// This function should use the matrix and viewport ratio previously set
    /// with `set_dimensions`.
    ///
    /// The convention is that if passed an identity matrix the widget must fill the entire
    /// viewport horizontally, vertically, or both.
    fn draw(&self) -> Vec<Shape>;

    /// Stores the information required to draw a widget.
    ///
    /// The convention is that if passed an identity matrix the widget must fill the entire
    /// viewport horizontally, vertically, or both.
    ///
    /// The viewport ratio is used for things that must not be skewed.
    ///
    /// The function returns a list of events to transmit to the parent.
    #[inline]
    fn set_dimensions(&mut self, matrix: &Matrix, viewport_height_per_width: f32)
                      -> Vec<Box<Event>>
    {
        vec![]
    }

    /// Tells the widget where the cursor is located over it.
    ///
    /// By default uses the return value of `draw()`.
    ///
    /// The function returns a list of events to transmit to the parent.
    #[inline]
    fn set_cursor(&mut self, _: Option<[f32; 2]>) -> Vec<Box<Event>> {
        vec![]
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
    pub fn apply_matrix(self, matrix: &Matrix) -> Shape {
        unimplemented!()
    }
}

/// Main struct of this library. Handles the UI as a whole.
pub struct Ui<T: ?Sized> where T: Widget {
    viewport_height_per_width: f32,
    widget: T,
}

impl<T> Ui<T> where T: Widget {
    /// Builds a new `Ui`.
    pub fn new(mut widget: T, viewport_height_per_width: f32) -> Ui<T> {
        widget.set_dimensions(&Matrix::identity(), viewport_height_per_width);

        Ui {
            viewport_height_per_width: viewport_height_per_width,
            widget: widget,
        }
    }
}

impl<T: ?Sized> Ui<T> where T: Widget {
    /// "Draws" the UI by returning a list of shapes. The list is ordered from bottom to top (in
    /// other words, shapes at the start of the list can be obstructed by shapes further ahead
    /// in the list).
    pub fn draw(&self) -> Vec<Shape> {
        self.widget.draw()
    }

    pub fn set_cursor(&mut self, cursor: Option<[f32; 2]>) {
        self.widget.set_cursor(cursor);
    }

    /// Returns a reference to the main widget stored in the object.
    #[inline]
    pub fn widget(&self) -> &T {
        &self.widget
    }

    /// Returns a mutable reference to the main widget stored in the object.
    #[inline]
    pub fn widget_mut(&mut self) -> &mut T {
        &mut self.widget
    }
}
