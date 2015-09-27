use Layout;
use Event;
use Matrix;
use Shape;
use Widget;

pub struct Image {
    name: String,
    height_per_width: f32,
    matrix: Option<Matrix>,
}

impl Image {
    #[inline]
    pub fn new<S>(name: S, height_per_width: f32) -> Image where S: Into<String> {
        Image {
            name: name.into(),
            height_per_width: height_per_width,
            matrix: None,
        }
    }

    #[inline]
    pub fn set_name<S>(&mut self, name: S) where S: Into<String> {
        self.name = name.into();
    }
}

impl Widget for Image {
    #[inline]
    fn build_layout(&self) -> Layout {
        let shape = Shape::Image { matrix: Matrix::identity(), name: self.name.clone() };
        Layout::Shapes(Box::new(shape))
    }
}
