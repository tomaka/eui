extern crate eui;

use std::sync::Arc;

#[test]
fn basic() {
    struct FullWidget;
    impl eui::Widget for FullWidget {
        fn build_layout(&self, _: f32, _: eui::Alignment) -> eui::Layout {
            let s = eui::Shape::Image { name: String::new(), matrix: eui::Matrix::identity() };
            eui::Layout::Shapes(vec![s])
        }
    }

    let ui = eui::Ui::new(FullWidget, 1.0);
    let shapes = ui.draw();
    assert_eq!(shapes,
               &[eui::Shape::Image { name: String::new(), matrix: eui::Matrix::identity() }]);
}

#[test]
fn horizontal_split_two() {
    struct FullWidget;
    impl eui::Widget for FullWidget {
        fn build_layout(&self, _: f32, _: eui::Alignment) -> eui::Layout {
            let s = eui::Shape::Image { name: String::new(), matrix: eui::Matrix::identity() };
            eui::Layout::Shapes(vec![s])
        }
    }

    struct TestedWidget;
    impl eui::Widget for TestedWidget {
        fn build_layout(&self, _: f32, _: eui::Alignment) -> eui::Layout {
            eui::Layout::HorizontalBar {
                alignment: eui::HorizontalAlignment::Center,
                vertical_align: false,
                children: vec![
                    eui::Child { child: Arc::new(FullWidget), weight: 1, collapse: false,
                                 alignment: Default::default(), padding_top: 0.0, padding_left: 0.0,
                                 padding_bottom: 0.0, padding_right: 0.0 },
                    eui::Child { child: Arc::new(FullWidget), weight: 1, collapse: false,
                                 alignment: Default::default(), padding_top: 0.0, padding_left: 0.0,
                                 padding_bottom: 0.0, padding_right: 0.0 },
                ],
            }
        }
    }

    let ui = eui::Ui::new(TestedWidget, 1.0);
    let shapes = ui.draw();
    assert_eq!(shapes,
               &[eui::Shape::Image { name: String::new(), matrix: eui::Matrix::translate(-0.5, 0.0) * eui::Matrix::scale_wh(0.5, 1.0) },
                 eui::Shape::Image { name: String::new(), matrix: eui::Matrix::translate(0.5, 0.0) * eui::Matrix::scale_wh(0.5, 1.0) }]);
}
