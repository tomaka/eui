use Alignment;
use Layout;
use Widget;

/// An empty widget.
pub struct Empty;

impl Empty {
    /// Builds an `Empty`.
    #[inline]
    pub fn new() -> Empty {
        Empty
    }
}

impl Widget for Empty {
    #[inline]
    fn build_layout(&self, _: f32, _: Alignment) -> Layout {
        Layout::Shapes(Vec::new())
    }
}
