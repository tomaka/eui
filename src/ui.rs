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

        let main_node = Node::new(state.clone() as Arc<_>, Matrix::identity(),
                                  viewport_height_per_width, alignment);

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

        *self.main_node.lock().unwrap() = Node::new(self.widget.clone(), Matrix::identity(),
                                                    viewport, alignment);

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

            *main_node = Node::new(self.widget.clone(), Matrix::identity(),
                                   viewport, alignment);
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
    matrix: Matrix,
    state: Arc<Widget>,
    children: Vec<Node>,
    shapes: Vec<Shape>,
    needs_rebuild: bool,

    // empty space around the widget in local coordinates
    empty_top: f32,
    empty_right: f32,
    empty_bottom: f32,
    empty_left: f32,
}

impl Node {
    fn new(state: Arc<Widget>, matrix: Matrix, viewport_height_per_width: f32,
           alignment: Alignment) -> Node
    {
        // TODO: take rotation into account for the height per width
        let my_height_per_width = viewport_height_per_width * matrix.0[1][1] / matrix.0[0][0];

        match state.build_layout(my_height_per_width, alignment) {
            Layout::AbsolutePositionned(list) => {
                // TODO: arbitrary alignment
                let children_alignment = Alignment {
                    horizontal: HorizontalAlignment::Center,
                    vertical: VerticalAlignment::Center,
                };

                let new_children: Vec<Node> = list.into_iter().map(|(m, w)| {
                    Node::new(w, m, viewport_height_per_width, children_alignment)
                }).collect();

                Node {
                    matrix: matrix,
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

            Layout::HorizontalBar { alignment, children } => {
                Node::with_layout(state, children, Alignment { horizontal: alignment, .. Default::default() },
                                  false, viewport_height_per_width, matrix)
            },

            Layout::VerticalBar { alignment, children } => {
                Node::with_layout(state, children, Alignment { vertical: alignment, .. Default::default() },
                                  true, viewport_height_per_width, matrix)
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
                    matrix: matrix,
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
                   viewport_height_per_width: f32, matrix: Matrix) -> Node
    {
        let mut empty_top = if vertical { 0.0 } else { 1.0 };
        let mut empty_right = if vertical { 1.0 } else { 0.0 };
        let mut empty_bottom = if vertical { 0.0 } else { 1.0 };
        let mut empty_left = if vertical { 1.0 } else { 0.0 };

        // sum of the weight of all children
        let elems_len = 1.0 / children.iter().fold(0, |a, b| a + b.weight) as f32;


        struct TempNode {
            node: Node,
            child: Child,
            position: [f32; 2],
            scale: [f32; 2],
            actual_content_percent: f32,
            inner_padding_matrix: Matrix,
        }

        let mut offset = 0;
        let mut children = children.into_iter().map(|child| {
            // position of the center of the child relative to the center of the current node
            let position = (2.0 * offset as f32 + child.weight as f32) * elems_len - 1.0;
            let position = if vertical { [0.0, position] } else { [position, 0.0] };

            // scale of the child relative to the current node
            let scale = if vertical {
                [1.0, child.weight as f32 * elems_len]
            } else {
                [child.weight as f32 * elems_len, 1.0]
            };

            // matrix containing the transformation to adjust for the padding
            let inner_padding_matrix = {
                let inner_position = Matrix::translate((child.padding_left - child.padding_right) * 0.5,
                                                       (child.padding_bottom - child.padding_top) * 0.5);
                let inner_scale = Matrix::scale_wh(1.0 - (child.padding_left + child.padding_right) * 0.5,
                                                   1.0 - (child.padding_bottom + child.padding_top) * 0.5);
                inner_position * inner_scale
            };

            let node = Node::new(child.child.clone(),
                                 Matrix::translate(position[0], position[1]) *
                                    Matrix::scale_wh(scale[0], scale[1]) * inner_padding_matrix,
                                 viewport_height_per_width, child.alignment);

            // percent of the child that actually contains stuff, relative to the current node
            let actual_content_percent = elems_len * child.weight as f32 * if child.collapse {
                if vertical {
                    (1.0 + child.padding_top * 0.5 + child.padding_bottom * 0.5 -
                        node.empty_bottom * 0.5 - node.empty_top * 0.5)
                } else {
                    (1.0 + child.padding_left * 0.5 + child.padding_right * 0.5 -
                        node.empty_left * 0.5 - node.empty_right * 0.5)
                }
            } else {
                1.0
            };

            offset += child.weight;

            if vertical {
                if node.empty_left - child.padding_left < empty_left { empty_left = node.empty_left - child.padding_left; }
                if node.empty_right - child.padding_right < empty_right { empty_right = node.empty_right - child.padding_right; }
            } else {
                if node.empty_top - child.padding_top < empty_top { empty_top = node.empty_top - child.padding_top; }
                if node.empty_bottom - child.padding_bottom < empty_bottom { empty_bottom = node.empty_bottom - child.padding_bottom; }
            }

            TempNode {
                node: node,
                child: child,
                position: position,
                scale: scale,
                actual_content_percent: actual_content_percent,
                inner_padding_matrix: inner_padding_matrix,
            }
        }).collect::<Vec<_>>();

        let real_len = 2.0 * children.iter().map(|tmp_node| tmp_node.actual_content_percent)
                                     .fold(0.0, |a, b| a + b);

        let start_offset = if vertical {
            match alignment.vertical {
                VerticalAlignment::Bottom => -1.0,
                VerticalAlignment::Center => -real_len * 0.5,
                VerticalAlignment::Top => 1.0 - real_len,
            }
        } else {
            match alignment.horizontal {
                HorizontalAlignment::Left => -1.0,
                HorizontalAlignment::Center => -real_len * 0.5,
                HorizontalAlignment::Right => 1.0 - real_len,
            }
        };

        let mut offset = start_offset;
        for tmp_node in children.iter_mut() {
            let position = offset + tmp_node.actual_content_percent;
            offset += tmp_node.actual_content_percent * 2.0;
            let position = if vertical { [0.0, position] } else { [position, 0.0] };

            tmp_node.node.matrix = Matrix::translate(position[0], position[1]) *
                                          Matrix::scale_wh(tmp_node.scale[0], tmp_node.scale[1]) *
                                          tmp_node.inner_padding_matrix;
        }

        if vertical {
            if let Some(c) = children.get(0) {
                if !c.child.collapse {
                    empty_bottom = (c.node.empty_bottom - c.child.padding_bottom) * c.child.weight as f32 * elems_len;
                }
            }

            if let Some(c) = children.last() {
                if !c.child.collapse {
                    empty_top = (c.node.empty_top - c.child.padding_top) * c.child.weight as f32 * elems_len;
                }
            }

        } else {
            if let Some(c) = children.get(0) {
                if !c.child.collapse {
                    empty_left = (c.node.empty_left - c.child.padding_left) * c.child.weight as f32 * elems_len;
                }
            }

            if let Some(c) = children.last() {
                if !c.child.collapse {
                    empty_right = (c.node.empty_right - c.child.padding_right) * c.child.weight as f32 * elems_len;
                }
            }
        }

        Node {
            matrix: matrix,
            state: state,
            children: children.into_iter().map(|child| child.node).collect(),
            shapes: Vec::new(),
            needs_rebuild: false,
            empty_top: empty_top,
            empty_right: empty_right,
            empty_bottom: empty_bottom,
            empty_left: empty_left,
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

        for child in &mut self.children {
            if child.needs_rebuild() {
                return true;
            }
        }

        false
    }

    fn build_shapes(&self) -> Vec<Shape> {
        let mut result = Vec::new();

        for c in &self.children {
            for s in c.build_shapes() { result.push(s.apply_matrix(&self.matrix)); }
        }

        for s in &self.shapes {
            result.push(s.clone().apply_matrix(&self.matrix));
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
    fn mouse_update(&mut self, mouse: Option<[f32; 2]>, parent_matrix: &Matrix,
                    new_mouse_down: bool, old_mouse_down: bool)
                    -> Vec<Box<Any>>
    {
        let mut result = Vec::new();

        {
            let mut events_for_self = Vec::new();

            for (num, child) in self.children.iter_mut().enumerate() {
                for ev in child.mouse_update(mouse, &(*parent_matrix * self.matrix), new_mouse_down,
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
            self.shapes.iter().find(|s| (*s).clone().apply_matrix(&self.matrix).apply_matrix(parent_matrix).hit_test(&mouse)).is_some()
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
