use Event;
use Matrix;
use Shape;
use Widget;

pub struct Image {
    name: String,
    matrix: Option<Matrix>,
}

impl Image {
    #[inline]
    pub fn new<S>(name: S) -> Image where S: Into<String> {
        Image {
            name: name.into(),
            matrix: None,
        }
    }

    #[inline]
    pub fn set_name<S>(&mut self, name: S) where S: Into<String> {
        self.name = name.into();
    }
}

impl Widget for Image {
    fn draw(&self) -> Vec<Shape> {
        if let Some(matrix) = self.matrix.clone() {
            let shape = Shape::Image { matrix: matrix, name: self.name.clone() };
            vec![shape]
        } else {
            vec![]
        }
    }

    #[inline]
    fn set_dimensions(&mut self, matrix: &Matrix, viewport_height_per_width: f32) -> Vec<Box<Event>> {
        self.matrix = Some(matrix.clone());
        vec![]
    }
}
