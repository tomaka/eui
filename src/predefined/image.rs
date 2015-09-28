use Alignment;
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
    ///
    ///
    /// The parameter `height_per_width` is the height per width ratio of the image. This ratio
    /// will be enforced when drawing.
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
    fn build_layout(&self, height_per_width: f32, alignment: Alignment) -> Layout {
        let matrix = if height_per_width > self.height_per_width {
            Matrix::scale_wh(1.0, self.height_per_width / height_per_width)
        } else {
            Matrix::scale_wh(height_per_width / self.height_per_width, 1.0)
        };

        let shape = Shape::Image { matrix: matrix, name: self.name.clone() };
        Layout::Shapes(vec![shape])
    }
}
