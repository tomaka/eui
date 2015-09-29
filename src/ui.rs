use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::mem;

use predefined;

use Alignment;
use HorizontalAlignment;
use Layout;
use Matrix;
use Shape;
use VerticalAlignment;
use Widget;

/// Main struct of this library. Handles the UI as a whole.
pub struct Ui<S> {
    viewport_height_per_width: Mutex<f32>,
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
            viewport_height_per_width: Mutex::new(viewport_height_per_width),
            widget: state,
            main_node: Mutex::new(main_node),
            hovering: AtomicBool::new(false),
            mouse_down: AtomicBool::new(false),
        }
    }

    /// Rebuilds the UI after the state has been changed.
    #[inline]
    pub fn rebuild(&self) {
        let viewport: f32 = self.viewport_height_per_width.lock().unwrap().clone();

        self.main_node.lock().unwrap().rebuild_children(viewport, Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        });

        // TODO: update mouse?
    }

    /// "Draws" the UI by returning a list of shapes. The list is ordered from bottom to top (in
    /// other words, shapes at the start of the list can be obstructed by shapes further ahead
    /// in the list).
    ///
    /// The matrices stored in the shapes assume that the viewport uses OpenGL coordinates. This
    /// means that the viewport has a width of 2, a height of 2, and that the origin is at the
    /// center of the screen.
    #[inline]
    pub fn draw(&self) -> Vec<Shape> {
        let viewport: f32 = self.viewport_height_per_width.lock().unwrap().clone();

        let mut main_node = self.main_node.lock().unwrap();

        if main_node.needs_rebuild() {
            main_node.rebuild_children(viewport, Alignment {
                horizontal: HorizontalAlignment::Center,
                vertical: VerticalAlignment::Center,
            });
        }

        main_node.build_shapes()
    }

    /// Changes the height per width ratio of the viewport and rebuilds the UI.
    #[inline]
    pub fn set_viewport_height_per_width(&self, value: f32) {
        let rebuild = {
            let mut viewport = self.viewport_height_per_width.lock().unwrap();
            if *viewport != value {
                *viewport = value;
                true
            } else {
                false
            }
        };

        if rebuild {
            self.rebuild();
        }
    }

    /// Sets the position and state of the cursor.
    ///
    /// This function will search for shapes that collide with the cursor and send mouse events
    /// to their owner.
    pub fn set_cursor(&self, cursor: Option<[f32; 2]>, down: bool) {
        let mut main_node = self.main_node.lock().unwrap();
        main_node.mouse_update(cursor, self.mouse_down.swap(down, Ordering::Relaxed), down);

        // FIXME: update "hovering"
    }

    /// Returns true if the mouse is hovering one of the elements of the UI.
    pub fn is_hovering(&self) -> bool {
        self.hovering.load(Ordering::Relaxed)
    }

    /// Returns a reference to the main widget stored in the object.
    ///
    /// Note that the UI won't be rebuilt after calling this function. You have to manually call
    /// the `rebuild()` method.
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

                let mut offset = 0;
                children.into_iter().map(|(weight, widget)| {
                    let position = (2.0 * offset as f32 + weight as f32) * elems_len - 1.0;
                    let position = Matrix::translate(position, 0.0);
                    let scale = Matrix::scale_wh(weight as f32 * elems_len, 1.0);

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

            Layout::VerticalBar { alignment, children } => {
                let children_alignment = Alignment {
                    horizontal: alignment,
                    vertical: VerticalAlignment::Center,
                };

                let elems_len = 1.0 / children.iter().fold(0, |a, b| a + b.0) as f32;

                let mut offset = 0;
                children.into_iter().map(|(weight, widget)| {
                    let position = (2.0 * offset as f32 + weight as f32) * elems_len - 1.0;
                    let position = Matrix::translate(0.0, position);
                    let scale = Matrix::scale_wh(1.0, weight as f32 * elems_len);

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

    /// Sends an event to the node and returns `true` if the event must be propagated to
    /// the parent.
    fn send_event(&mut self, event: &Any, child_num: Option<usize>) -> bool {
        let outcome = self.state.handle_event(event, child_num);

        if outcome.refresh_layout {
            self.needs_rebuild = true;
        }

        outcome.propagate_to_parent
    }

    /// Sends mouse events to the node, and returns a list of events that must be propagated to the
    /// parent.
    fn mouse_update(&mut self, mouse: Option<[f32; 2]>, new_mouse_down: bool, old_mouse_down: bool)
                    -> Vec<Box<Any>>
    {
        let mut result = Vec::new();

        {
            let mut events_for_self = Vec::new();

            for (num, child) in self.children.iter_mut().enumerate() {
                for ev in child.mouse_update(mouse, new_mouse_down, old_mouse_down) {
                    events_for_self.push((ev, num));
                }

                // TODO: break if event handled
            }

            for (ev, child) in events_for_self {
                if self.send_event(&*ev, Some(child)) {
                    result.push(ev);
                }
            }
        }

        let hit = if let Some(mouse) = mouse {
            self.shapes.iter().find(|s| s.hit_test(&mouse)).is_some()
        } else {
            false
        };

        // TODO: do not send these events if not necessary (eg. do not send mouse leave if mouse
        // wasn't over the element)
        if hit {
            let ev = Box::new(predefined::MouseEnterEvent) as Box<Any>;
            if self.send_event(&*ev, None) {
                result.push(ev);
            }

        } else {
            let ev = Box::new(predefined::MouseLeaveEvent) as Box<Any>;
            if self.send_event(&*ev, None) {
                result.push(ev);
            }
        };

        if hit && !new_mouse_down && old_mouse_down {
            let ev = Box::new(predefined::MouseClick) as Box<Any>;
            if self.send_event(&*ev, None) {
                result.push(ev);
            }
        }

        result
    }
}
