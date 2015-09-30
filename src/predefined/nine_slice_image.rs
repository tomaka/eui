use Alignment;
use Layout;
use Matrix;
use Widget;

pub struct NineSliceImage {
    corner_image: String,
    border_image: String,
    background_image: String,
    border_width: f32,
    border_height: f32,
}

impl NineSliceImage {
    #[inline]
    pub fn new<S1, S2, S3>(corner_image: S1, border_image: S2, background_image: S3,
                           border_width: f32, border_height: f32) -> NineSliceImage
                           where S1: Into<String>, S2: Into<String>, S3: Into<String>
    {
        NineSliceImage {
            corner_image: corner_image.into(),
            border_image: border_image.into(),
            background_image: background_image.into(),
            border_width: border_width,
            border_height: border_height,
        }
    }
}

impl Widget for NineSliceImage {
    #[inline]
    fn build_layout(&self, height_per_width: f32, _: Alignment) -> Layout {
        let _corner_scale = if height_per_width > 1.0 {
            Matrix::scale_wh(height_per_width * self.border_width, self.border_height)
        } else {
            Matrix::scale_wh(self.border_width, self.border_height / height_per_width)
        };

        unimplemented!()        // TODO: 

        //let vertical_border_scale = ;
    }
}
