use Children;
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
    fn build_layout(&self) -> Children {
        let shape = Shape::Image { matrix: Matrix::identity(), name: self.name.clone() };
        Children::Shapes(Box::new(shape))
    }
}
