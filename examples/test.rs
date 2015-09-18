extern crate eui;

struct SpellIcon {
    button: eui::predefined::Button
}

impl eui::Widget for SpellIcon {
    #[inline]
    fn draw(&self) -> Vec<eui::Shape> {
        self.button.draw()
    }

    #[inline]
    fn set_dimensions(&mut self, matrix: &eui::Matrix, viewport_height_per_width: f32)
                      -> Vec<Box<eui::Event>>
    {
        self.button.set_dimensions(matrix, viewport_height_per_width)
    }

    #[inline]
    fn set_cursor(&mut self, cursor: Option<[f32; 2]>) -> Vec<Box<eui::Event>> {
        self.button.set_cursor(cursor)
    }
}

struct SpellsBar {
    icons: Vec<SpellIcon>
}

impl eui::Widget for SpellsBar {
    #[inline]
    fn draw(&self) -> Vec<eui::Shape> {
        eui::layout::horizontal::draw(self.icons.iter().map(|w| w as &_))
    }

    #[inline]
    fn set_dimensions(&mut self, matrix: &eui::Matrix, viewport_height_per_width: f32)
                      -> Vec<Box<eui::Event>>
    {
        eui::layout::horizontal::set_dimensions(self.icons.iter_mut().map(|w| w as &mut _),
                                                matrix, viewport_height_per_width)
    }

    #[inline]
    fn set_cursor(&mut self, _: Option<[f32; 2]>) -> Vec<Box<eui::Event>> {
        vec![]
    }
}

fn main() {
    let bar = SpellsBar { icons: vec![SpellIcon { button: eui::predefined::Button::new("normal", "hovered") }] };
    let ui = eui::Ui::new(bar, 1.0);

    println!("{:?}", ui.draw());
}
