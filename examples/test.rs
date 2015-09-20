extern crate eui;

use std::sync::Arc;

struct SpellIcon {
    button: eui::predefined::Button
}

impl eui::Widget for SpellIcon {
    fn build_layout(&self) -> eui::Layout {
        self.button.build_layout()
    }
}

struct SpellsBar {
    icons: Vec<Arc<SpellIcon>>
}

impl eui::Widget for SpellsBar {
    fn build_layout(&self) -> eui::Layout {
        eui::Layout::HorizontalBar(self.icons.iter().map(|s| s.clone() as Arc<_>).collect())
    }
}

fn main() {
    let bar = SpellsBar { icons: vec![Arc::new(SpellIcon { button: eui::predefined::Button::new("normal", "hovered") })] };
    let ui = eui::Ui::new(bar, 1.0);

    println!("{:?}", ui.draw());
}
