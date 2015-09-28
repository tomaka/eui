use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::mem;

use predefined;

use Alignment;
use EventOutcome;
use HorizontalAlignment;
use Layout;
use Matrix;
use Shape;
use VerticalAlignment;
use Widget;

/// Main struct of this library. Handles the UI as a whole.
pub struct Ui<S> {
    viewport_height_per_width: f32,
    widget: Arc<S>,
    main_node: Mutex<Node>,
    hovering: AtomicBool,
    mouse_down: AtomicBool,
}

impl<S> Ui<S> where S: Widget {
    /// Builds a new `Ui`.
    pub fn new(state: S, viewport_height_per_width: f32) -> Ui<S> {
        let state = Arc::new(state);

        let mut main_node = Node {
            matrix: Matrix::identity(),
            state: state.clone() as Arc<_>,
            children: Vec::new(),
            shapes: Vec::new(),
            needs_rebuild: false,
        };

        main_node.rebuild_children(viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        Ui {
            viewport_height_per_width: viewport_height_per_width,
            widget: state,
            main_node: Mutex::new(main_node),
            hovering: AtomicBool::new(false),
            mouse_down: AtomicBool::new(false),
        }
    }

    /// Rebuilds the UI after the state has been changed.
    #[inline]
    pub fn rebuild(&self) {
        self.main_node.lock().unwrap().rebuild_children(self.viewport_height_per_width, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        // TODO: update mouse?
    }

    /// "Draws" the UI by returning a list of shapes. The list is ordered from bottom to top (in
    /// other words, shapes at the start of the list can be obstructed by shapes further ahead
    /// in the list).
    #[inline]
    pub fn draw(&self) -> Vec<Shape> {
        let mut main_node = self.main_node.lock().unwrap();

        if main_node.needs_rebuild() {
            main_node.rebuild_children(self.viewport_height_per_width, Alignment {
                horizontal: HorizontalAlignment::Center,
                vertical: VerticalAlignment::Center,
            });
        }

        main_node.build_shapes()
    }

    /// Changes the height per width ratio of the viewport and rebuilds the UI.
    // TODO: use &self not &mut self
    #[inline]
    pub fn set_viewport_height_per_width(&mut self, value: f32) {
        if self.viewport_height_per_width != value {
            self.viewport_height_per_width = value;
            self.rebuild();
        }
    }

    pub fn set_cursor(&self, cursor: Option<[f32; 2]>, down: bool) {
        let mut main_node = self.main_node.lock().unwrap();

        let outcome = main_node.mouse_update(cursor, self.mouse_down.swap(down, Ordering::Relaxed),
                                             down);
        if outcome.refresh_layout {
            main_node.needs_rebuild = true;
        }

        // FIXME:
        main_node.needs_rebuild = true;

        // FIXME: update "hovering"
    }

    /// Returns true if the mouse is hovering one of the elements of the UI.
    pub fn is_hovering(&self) -> bool {
        self.hovering.load(Ordering::Relaxed)
    }

    /// Returns a reference to the main widget stored in the object.
    #[inline]
    pub fn widget(&self) -> &S {
        &self.widget
    }
}

struct Node {
    /// Absolute matrix (relative to root)
    matrix: Matrix,
    state: Arc<Widget>,
    children: Vec<Node>,
    shapes: Vec<Shape>,
    needs_rebuild: bool,
}

impl Node {
    #[inline]
    fn needs_rebuild(&mut self) -> bool {
        if self.needs_rebuild {
            self.needs_rebuild = false;
            return true;
        }

        if self.state.needs_rebuild() {
            return true;
        }

        for child in &mut self.children {
            if child.needs_rebuild() {
                return true;
            }
        }

        false
    }

    fn rebuild_children(&mut self, viewport_height_per_width: f32, alignment: Alignment) {
        // TODO: take rotation into account for the height per width
        let my_height_per_width = viewport_height_per_width * self.matrix.0[1][1] / self.matrix.0[0][0];
        let mut state_children = self.state.build_layout(my_height_per_width, alignment);

        self.shapes = match state_children {
            Layout::Shapes(ref mut look) => {
                let shapes = mem::replace(look, Vec::new());
                shapes.into_iter().map(|s| s.apply_matrix(&self.matrix)).collect()
            },
            _ => Vec::new()
        };

        self.children = match state_children {
            Layout::AbsolutePositionned(list) => {
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: VerticalAlignment::Center,
                };

                list.into_iter().map(|(matrix, w)| {
                    let mut node = Node {
                        matrix: self.matrix.clone() * matrix,
                        state: w, 
                        children: Vec::new(),
                        shapes: Vec::new(),
                        needs_rebuild: false,
                    };

                    node.rebuild_children(viewport_height_per_width, children_alignment);
                    node
                }).collect()
            },

            Layout::HorizontalBar { alignment, children } => {
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: alignment,
                };

                let elems_len = 1.0 / children.iter().fold(0, |a, b| a + b.0) as f32;
                let scale = Matrix::scale_wh(elems_len, 1.0);

                let mut offset = 0;
                children.into_iter().map(|(weight, widget)| {
                    let position = (offset as f32 * 2.0 + 1.0) * elems_len - 1.0;
                    let position = Matrix::translate(position, 0.0);

                    offset += weight;

                    let mut node = Node {
                        matrix: self.matrix * position * scale,
                        state: widget,
                        children: Vec::new(),
                        shapes: Vec::new(),
                        needs_rebuild: false,
                    };

                    node.rebuild_children(viewport_height_per_width, children_alignment);
                    node
                }).collect()
            },

            Layout::Shapes(_) => Vec::new(),

            _ => unimplemented!()
        };

        self.needs_rebuild = false;
    }

    fn build_shapes(&self) -> Vec<Shape> {
        let mut result = Vec::new();

        for c in &self.children {
            for s in c.build_shapes() { result.push(s); }
        }

        for s in &self.shapes {
            result.push(s.clone());
        }

        result
    }

    fn send_event(&self, event: Box<Any>) -> EventOutcome {
        self.state.handle_event(event)
    }

    fn mouse_update(&mut self, mouse: Option<[f32; 2]>, new_mouse_down: bool, old_mouse_down: bool)
                    -> EventOutcome
    {
        for child in &mut self.children {
            let outcome = child.mouse_update(mouse, new_mouse_down, old_mouse_down);

            if outcome.refresh_layout {
                child.needs_rebuild = true;
            }

            if outcome.propagate_to_parent {
                // TODO: implement
            }

            // TODO: break if event handled
        }

        let hit = if let Some(mouse) = mouse {
            self.shapes.iter().find(|s| s.hit_test(&mouse)).is_some()
        } else {
            false
        };

        // TODO: do not send these events twice
        if hit {
            self.send_event(Box::new(predefined::MouseEnterEvent))
        } else {
            self.send_event(Box::new(predefined::MouseLeaveEvent))
        };

        if !new_mouse_down && old_mouse_down {
            self.send_event(Box::new(predefined::MouseClick));
        }

        // FIXME: wrong
        Default::default()
    }
}
