extern crate eui;

struct CharacterSheet {
    val: i32
}

impl eui::Widget for CharacterSheet {
    fn draw(&self) -> Vec<eui::Shape> {
        unimplemented!()
    }
}

struct SpellIcon {
    val: i32
}

impl eui::Widget for SpellIcon {
    fn draw(&self) -> Vec<eui::Shape> {
        unimplemented!()
    }
}

struct SpellsBar {
    icons: Vec<SpellIcon>
}

impl eui::Widget for SpellsBar {
    fn draw(&self) -> Vec<eui::Shape> {
        unimplemented!()
    }
}

fn main() {
    let bar = SpellsBar { icons: vec![SpellIcon { val: 3 }] };
    let ui = eui::Ui::new(bar, 1.0);

    println!("{:?}", ui.draw());
}
