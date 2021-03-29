mod canvas;

pub use canvas::PenInput;
use canvas::{Canvas, SingleVecImage, RGB};
use std::{cell::RefCell, rc::Rc};

struct ViewportData {
    size: (usize, usize),
    background_color: RGB,
    cairo_context: Rc<RefCell<Option<cairo::Context>>>,
    canvas_position: (f64, f64),
}

pub struct Viewport {
    data: Rc<RefCell<ViewportData>>,
    canvas: Canvas,
    draw_handler: Box<dyn Fn()>,
}

fn make_draw_handler(
    viewport_data: Rc<RefCell<ViewportData>>,
) -> Box<dyn Fn(&SingleVecImage, (usize, usize))> {
    Box::new(move |image: &SingleVecImage, canvas_size: (usize, usize)| {
        let stride = cairo::Format::Rgb24
            .stride_for_width(image.width as u32)
            .unwrap();
        let image = cairo::ImageSurface::create_for_data(
            image.vector.clone(),
            cairo::Format::Rgb24,
            canvas_size.0 as i32,
            canvas_size.1 as i32,
            stride,
        )
        .unwrap();
        let viewport_data = viewport_data.borrow();
        let context = viewport_data.cairo_context.borrow();
        let context = context.as_ref().unwrap();
        let c = viewport_data.background_color;
        context.set_source_rgb(
            c.r() as f64 / 0xff as f64,
            c.g() as f64 / 0xff as f64,
            c.b() as f64 / 0xff as f64,
        );
        context.paint();
        let (x, y) = viewport_data.canvas_position;
        context.set_source_surface(&image, x, y);
        context.paint();
    })
}

impl Viewport {
    pub fn new(
        cairo_context: Rc<RefCell<Option<cairo::Context>>>,
        size: (usize, usize),
        draw_handler: Box<dyn Fn()>,
    ) -> Viewport {
        let data = Rc::new(RefCell::new(ViewportData {
            size,
            background_color: RGB::new(0x33, 0x33, 0x33),
            cairo_context: cairo_context.clone(),
            canvas_position: (0.0, 0.0),
        }));
        Viewport {
            data: data.clone(),
            canvas: Canvas::new(make_draw_handler(data), (500, 500)),
            draw_handler,
        }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        let PenInput { x, y, pressure } = input;
        let canvas_position = self.data.borrow().canvas_position;
        self.canvas.pen_stroke(PenInput {
            x: x - canvas_position.0,
            y: y - canvas_position.1,
            pressure,
        });
        (self.draw_handler)();
    }

    pub fn pen_stroke_end(&mut self) {
        self.canvas.pen_stroke_end()
    }

    pub fn reflect_all(&mut self) {
        self.canvas.reflect_all();
        (self.draw_handler)();
    }

    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        let mut data = self.data.borrow_mut();
        data.size = (width, height);
        data.canvas_position = (
            (width as f64 - self.canvas.get_size().0 as f64) / 2.0,
            (height as f64 - self.canvas.get_size().1 as f64) / 2.0,
        );
    }
}
