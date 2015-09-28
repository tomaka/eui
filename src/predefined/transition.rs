use std::cmp;
use std::ops::Deref;
use std::sync::Arc;
use time;

use Alignment;
use Event;
use Layout;
use Matrix;
use Shape;
use Widget;

pub struct Transition<W> {
    child: Arc<W>,
    anim_start_ns: u64,
    anim_duration_ns: u64,
}

impl<W> Transition<W> where W: Widget {
    pub fn new(child: Arc<W>) -> Transition<W> {
        // TODO: allow customization
        Transition {
            child: child,
            anim_start_ns: time::precise_time_ns() + 1000000000,
            anim_duration_ns: 3 * 1000000000,       // 3s
        }
    }
}

impl<W> Widget for Transition<W> where W: Widget {
    fn build_layout(&self, _: f32, _: Alignment) -> Layout {
        let anim_progress = time::precise_time_ns().saturating_sub(self.anim_start_ns);
        let anim_progress = anim_progress as f32 / self.anim_duration_ns as f32;
        let anim_progress = if anim_progress > 1.0 { 1.0 } else { anim_progress };

        let matrix = Matrix::translate((-anim_progress * 10.0).exp(), 0.0);

        Layout::AbsolutePositionned(vec![
            (matrix, self.child.clone())
        ])
    }

    #[inline]
    fn needs_rebuild(&self) -> bool {
        let in_progress = time::precise_time_ns() < self.anim_start_ns + self.anim_duration_ns;
        in_progress || self.child.needs_rebuild()
    }
}

impl<W> Deref for Transition<W> {
    type Target = W;

    fn deref(&self) -> &W {
        &self.child
    }
}
