use Layout;
use Event;
use Matrix;
use Shape;
use Widget;

pub struct Label {
    dummy: ()
}

impl Label {
    /// Initializes a new label.
    #[inline]
    pub fn new<S>(text: S) -> Label
                  where S: Into<String>
    {
        Label {
            dummy: ()
        }
    }

    #[inline]
    pub fn set_text<S>(&mut self, text: S) where S: Into<String> {

    }
}

impl Widget for Label {
    #[inline]
    fn build_layout(&self) -> Layout {
        Layout::HorizontalBar { alignment: ::Alignment::Center, children: vec![] }
    }
}
