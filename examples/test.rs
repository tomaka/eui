extern crate eui;

use std::sync::Arc;

struct SpellIcon {
    button: eui::predefined::Button
}

impl eui::Widget for SpellIcon {
    fn build_children(&self) -> eui::Children {
        self.button.build_children()
    }
}

struct SpellsBar {
    icons: Vec<Arc<SpellIcon>>
}

impl eui::Widget for SpellsBar {
    fn build_children(&self) -> eui::Children {
        eui::Children::HorizontalBar(self.icons.iter().map(|s| s.clone() as Arc<_>).collect())
    }
}

fn main() {
    let bar = SpellsBar { icons: vec![Arc::new(SpellIcon { button: eui::predefined::Button::new("normal", "hovered") })] };
    let ui = eui::Ui::new(bar, 1.0);

    println!("{:?}", ui.draw());
}
