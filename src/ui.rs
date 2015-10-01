use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::mem;

use predefined;

use Alignment;
use Child;
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

        let alignment = Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        };

        let main_node = Node::new(state.clone() as Arc<_>, viewport_height_per_width, alignment);

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

        let alignment = Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        };

        *self.main_node.lock().unwrap() = Node::new(self.widget.clone(), viewport, alignment);

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
            let alignment = Alignment {
                horizontal: HorizontalAlignment::Center,
                vertical: VerticalAlignment::Center,
            };

            *main_node = Node::new(self.widget.clone(), viewport, alignment);
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
        main_node.mouse_update(cursor, &Matrix::identity(),
                               self.mouse_down.swap(down, Ordering::Relaxed), down);

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
    /// Local matrix
    state: Arc<Widget>,
    children: Vec<(Matrix, Node)>,
    shapes: Vec<Shape>,
    needs_rebuild: bool,

    // empty space around the widget in local coordinates
    empty_top: f32,
    empty_right: f32,
    empty_bottom: f32,
    empty_left: f32,
}

impl Node {
    fn new(state: Arc<Widget>, my_height_per_width: f32, alignment: Alignment) -> Node {
        match state.build_layout(my_height_per_width, alignment) {
            Layout::AbsolutePositionned(list) => {
                // TODO: arbitrary alignment
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: VerticalAlignment::Center,
                };

                let new_children: Vec<(Matrix, Node)> = list.into_iter().map(|(m, w)| {
                    (m, Node::new(w, my_height_per_width, children_alignment))
                }).collect();

                Node {
                    state: state,
                    children: new_children,
                    shapes: Vec::new(),
                    needs_rebuild: false,
                    empty_top: 0.0,
                    empty_right: 0.0,
                    empty_bottom: 0.0,
                    empty_left: 0.0,
                }
            },

            Layout::HorizontalBar { alignment, children, vertical_align } => {
                Node::with_layout(state, children, Alignment { horizontal: alignment, .. Default::default() },
                                  false, my_height_per_width, vertical_align)
            },

            Layout::VerticalBar { alignment, children, horizontal_align } => {
                Node::with_layout(state, children, Alignment { vertical: alignment, .. Default::default() },
                                  true, my_height_per_width, horizontal_align)
            },

            Layout::Shapes(shapes) => {
                let mut empty_top = 1.0;
                let mut empty_right = 1.0;
                let mut empty_bottom = 1.0;
                let mut empty_left = 1.0;

                let shapes = shapes.into_iter().map(|shape| {
                    let (t, r, b, l) = shape.get_bounding_box();
                    let t = 1.0 - t; let r = 1.0 - r; let b = b + 1.0; let l = l + 1.0;
                    if t < empty_top { empty_top = t; }
                    if r < empty_right { empty_right = r; }
                    if b < empty_bottom { empty_bottom = b; }
                    if l < empty_left { empty_left = l; }

                    shape
                }).collect::<Vec<_>>();

                Node {
                    state: state,
                    children: Vec::new(),
                    shapes: shapes,
                    needs_rebuild: false,
                    empty_top: empty_top,
                    empty_right: empty_right,
                    empty_bottom: empty_bottom,
                    empty_left: empty_left,
                }
            }
        }
    }

    fn with_layout(state: Arc<Widget>, children: Vec<Child>, alignment: Alignment, vertical: bool,
                   my_height_per_width: f32, other_align: bool) -> Node
    {
        // In this function, the word "flow" designates the dimension that is being operated and
        // "perpendicular" designates the other dimension. If `vertical` is true, then the flow
        // is the y dimension and the perpendicular dimension is x.

        // inverse of the sum of the weight of all children
        let weight_sum_inverse = 1.0 / children.iter().fold(0, |a, b| a + b.weight) as f32;

        // the first step is to build the children nodes
        let children: Vec<_> = children.into_iter().map(|child| {
            // calculating the height per width of the child
            let height_per_width = my_height_per_width * if vertical {
                child.weight as f32 * weight_sum_inverse
            } else {
                1.0 / (weight_sum_inverse * child.weight as f32)
            };

            // building its node
            let node = Node::new(child.child.clone(), height_per_width, child.alignment);
            (child, node)
        }).collect();

        // if `Some`, then the effective content of the perpendicular dimension must be this
        // given percentage
        let required_effective_perp_percentage = if other_align {
            let val = 1.0 / children.iter().map(|&(ref child, ref node)| {
                let flow_percent = child.weight as f32 * weight_sum_inverse * 0.5 * (2.0 - if child.collapse {
                    if vertical {
                        node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                    } else {
                        node.empty_left + node.empty_right - child.padding_left - child.padding_right
                    }
                } else {
                    0.0
                });

                let perp_percent = 0.5 * (2.0 - if vertical {
                    node.empty_left + node.empty_right - child.padding_left - child.padding_right
                } else {
                    node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                });

                flow_percent / perp_percent
            }).fold(0.0, |a, b| a + b);

            let val = if val >= 1.0 { 1.0 } else { val };
            Some(val)

        } else {
            None
        };

        // percentage of the widget (in the direction of the flow) that is effectively filled
        // with content
        let flow_effective_percentage = children.iter().map(|&(ref child, ref node)| {
            // the ratio to multiply the scale of the node with
            let scale_ratio = if let Some(req_perp) = required_effective_perp_percentage {
                // the percentage of the perpendicular dimension that is effectively filled with
                // content
                let effective_perp_percentage = 0.5 * (2.0 - if vertical {
                    node.empty_left + node.empty_right - child.padding_left - child.padding_right
                } else {
                    node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                });

                req_perp / effective_perp_percentage
            } else {
                1.0
            };

            let flow_empty = if child.collapse {
                if vertical {
                    node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                } else {
                    node.empty_left + node.empty_right - child.padding_left - child.padding_right
                }
            } else {
                0.0
            };

            (2.0 - flow_empty) * 0.5 * child.weight as f32 * weight_sum_inverse * scale_ratio
        }).fold(0.0, |a, b| a + b);

        // position of the left or bottom border of the first element
        let flow_start_border_position = if vertical {
            match alignment.vertical {
                VerticalAlignment::Bottom => -1.0,
                VerticalAlignment::Center => -flow_effective_percentage,
                VerticalAlignment::Top => 1.0 - flow_effective_percentage * 2.0,
            }
        } else {
            match alignment.horizontal {
                HorizontalAlignment::Left => -1.0,
                HorizontalAlignment::Center => -flow_effective_percentage,
                HorizontalAlignment::Right => 1.0 - flow_effective_percentage * 2.0,
            }
        };


        // now we iterate over each child and calculate its data

        let mut my_empty_top = if vertical { 0.0 } else { 1.0 };
        let mut my_empty_right = if vertical { 1.0 } else { 0.0 };
        let mut my_empty_bottom = if vertical { 0.0 } else { 1.0 };
        let mut my_empty_left = if vertical { 1.0 } else { 0.0 };

        let mut flow_current_border_position = flow_start_border_position;
        let num_children = children.len();
        let children: Vec<_> = children.into_iter().enumerate().map(|(child_num, (child, node))| {
            // the ratio to multiply the scale of the node with
            let scale_ratio = if let Some(req_perp) = required_effective_perp_percentage {
                // the percentage of the perpendicular dimension that is effectively filled with
                // content
                let effective_perp_percentage = 0.5 * (2.0 - if vertical {
                    node.empty_left + node.empty_right - child.padding_left - child.padding_right
                } else {
                    node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                });

                req_perp / effective_perp_percentage
            } else {
                1.0
            };

            // matrix containing the transformation to adjust for the padding
            let inner_padding_matrix = {
                let inner_position = Matrix::translate((child.padding_left - child.padding_right) * 0.5,
                                                       (child.padding_bottom - child.padding_top) * 0.5);
                let inner_scale = Matrix::scale_wh(1.0 - (child.padding_left + child.padding_right) * 0.5,
                                                   1.0 - (child.padding_bottom + child.padding_top) * 0.5);
                inner_position * inner_scale
            };

            // percentage of the total flow of the widget to be filled by this child
            let flow_percent = child.weight as f32 * weight_sum_inverse * 0.5 * (2.0 - if child.collapse {
                if vertical {
                    node.empty_top + node.empty_bottom - child.padding_top - child.padding_bottom
                } else {
                    node.empty_left + node.empty_right - child.padding_left - child.padding_right
                }
            } else {
                0.0
            });

            // adjusting the `my_empty_*` variables
            if vertical {
                if node.empty_left - child.padding_left < my_empty_left { my_empty_left = node.empty_left - child.padding_left; }
                if node.empty_right - child.padding_right < my_empty_right { my_empty_right = node.empty_right - child.padding_right; }
                if child_num == 0 {
                    if !child.collapse {
                        my_empty_bottom = (node.empty_bottom - child.padding_bottom) * child.weight as f32 * weight_sum_inverse;
                    }
                }
                if child_num == num_children - 1 {
                    if !child.collapse {
                        my_empty_top = (node.empty_top - child.padding_top) * child.weight as f32 * weight_sum_inverse;
                    }
                }
            } else {
                if node.empty_top - child.padding_top < my_empty_top { my_empty_top = node.empty_top - child.padding_top; }
                if node.empty_bottom - child.padding_bottom < my_empty_bottom { my_empty_bottom = node.empty_bottom - child.padding_bottom; }
                if child_num == 0 {
                    if !child.collapse {
                        my_empty_left = (node.empty_left - child.padding_left) * child.weight as f32 * weight_sum_inverse;
                    }
                }
                if child_num == num_children - 1 {
                    if !child.collapse {
                        my_empty_right = (node.empty_right - child.padding_right) * child.weight as f32 * weight_sum_inverse;
                    }
                }
            }

            // position of the center of this child in the flow
            let flow_center_position = flow_current_border_position + flow_percent * scale_ratio;
            flow_current_border_position += flow_percent * scale_ratio * 2.0;

            // matrix containing the position of this child
            let position_matrix = if vertical {
                Matrix::translate(0.0, flow_center_position)
            } else {
                Matrix::translate(flow_center_position, 0.0)
            };

            // matrix containing the scale of this child
            let scale_matrix = if vertical {
                Matrix::scale_wh(scale_ratio, scale_ratio * child.weight as f32 * weight_sum_inverse)
            } else {
                Matrix::scale_wh(scale_ratio * child.weight as f32 * weight_sum_inverse, scale_ratio)
            };

            // the total matrix for this child
            let total_matrix = position_matrix * scale_matrix * inner_padding_matrix;

            (total_matrix, node)
        }).collect();

        Node {
            state: state,
            children: children,
            shapes: Vec::new(),
            needs_rebuild: false,
            empty_top: my_empty_top,
            empty_right: my_empty_right,
            empty_bottom: my_empty_bottom,
            empty_left: my_empty_left,
        }
    }

    #[inline]
    fn needs_rebuild(&mut self) -> bool {
        if self.needs_rebuild {
            self.needs_rebuild = false;
            return true;
        }

        if self.state.needs_rebuild() {
            return true;
        }

        for &mut (_, ref mut child) in &mut self.children {
            if child.needs_rebuild() {
                return true;
            }
        }

        false
    }

    fn build_shapes(&self) -> Vec<Shape> {
        let mut result = Vec::new();

        for &(ref m, ref c) in &self.children {
            for s in c.build_shapes() { result.push(s.apply_matrix(m)); }
        }

        for s in &self.shapes {
            result.push(s.clone());
        }

        result
    }

    /// Sends an event to the node and returns events to propagate to the parent.
    fn send_event(&mut self, event: Box<Any>, child_num: Option<usize>) -> Vec<Box<Any>> {
        let outcome = self.state.handle_event(&*event, child_num);

        if outcome.refresh_layout {
            self.needs_rebuild = true;
        }

        let mut result = outcome.events_for_parent;
        if outcome.propagate_to_parent {
            result.push(event);
        }
        result
    }

    /// Sends mouse events to the node, and returns a list of events that must be propagated to the
    /// parent.
    fn mouse_update(&mut self, mouse: Option<[f32; 2]>, matrix: &Matrix, new_mouse_down: bool,
                    old_mouse_down: bool) -> Vec<Box<Any>>
    {
        let mut result = Vec::new();

        {
            let mut events_for_self = Vec::new();

            for (num, &mut (ref child_matrix, ref mut child)) in self.children.iter_mut().enumerate() {
                for ev in child.mouse_update(mouse, &(*matrix * *child_matrix), new_mouse_down,
                                             old_mouse_down)
                {
                    events_for_self.push((ev, num));
                }

                // TODO: break if event handled
            }

            for (ev, child) in events_for_self {
                for ev in self.send_event(ev, Some(child)) {
                    result.push(ev);
                }
            }
        }

        let hit = if let Some(mouse) = mouse {
            self.shapes.iter().find(|s| (*s).clone().apply_matrix(matrix).hit_test(&mouse)).is_some()
        } else {
            false
        };

        // TODO: do not send these events if not necessary (eg. do not send mouse leave if mouse
        // wasn't over the element)
        if hit {
            let ev = Box::new(predefined::MouseEnterEvent) as Box<Any>;
            for ev in self.send_event(ev, None) {
                result.push(ev);
            }

        } else {
            let ev = Box::new(predefined::MouseLeaveEvent) as Box<Any>;
            for ev in self.send_event(ev, None) {
                result.push(ev);
            }
        };

        if hit && !new_mouse_down && old_mouse_down {
            let ev = Box::new(predefined::MouseClick) as Box<Any>;
            for ev in self.send_event(ev, None) {
                result.push(ev);
            }
        }

        result
    }
}
