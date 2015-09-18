use Event;
use Matrix;
use Shape;
use Widget;

pub fn draw<'a, I>(list: I) -> Vec<Shape> where I: IntoIterator<Item = &'a Widget> {
    list.into_iter().flat_map(|w| w.draw().into_iter()).collect()
}

pub fn set_dimensions<'a, I>(list: I, matrix: &Matrix, viewport_height_per_width: f32)
                             -> Vec<Box<Event>>
                             where I: IntoIterator<Item = &'a mut Widget>
{
    let list = list.into_iter().collect::<Vec<_>>();
    let num = list.len();

    let elem_width = 1.0 / num as f32;

    let mut events = Vec::new();
    for (offset, elem) in list.into_iter().enumerate() {
        let e = elem.set_dimensions(&(*matrix * Matrix::scale_wh(elem_width, 1.0) *
                                      Matrix::translate(offset as f32 * elem_width, 0.0)),
                                    viewport_height_per_width);
        for e in e { events.push(e); }
    }
    events
}

pub fn set_cursor(cursor: Option<(f32, f32)>) -> Vec<Box<Event>> {
    vec![]
}
