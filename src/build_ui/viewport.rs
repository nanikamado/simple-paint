pub(crate) mod canvas;
use std::{cell::RefCell, rc::Rc};

use canvas::*;

pub struct Viewport {
    size: (usize, usize),
    background_color: RGB,
    cairo_context: Rc<RefCell<Option<cairo::Context>>>,
    canvas: Canvas,
    draw_handler: Box<dyn Fn()>,
}

fn make_draw_handler(
    cairo_context: Rc<RefCell<Option<cairo::Context>>>,
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
        let context = cairo_context.borrow();
        let context = context.as_ref().unwrap();
        context.set_source_rgb(0.6, 0.6, 0.6);
        context.paint();
        context.set_source_surface(&image, 0.0, 0.0);
        context.paint();
    })
}

impl Viewport {
    pub fn new(
        cairo_context: Rc<RefCell<Option<cairo::Context>>>,
        size: (usize, usize),
        draw_handler: Box<dyn Fn()>,
    ) -> Viewport {
        Viewport {
            size,
            background_color: RGB::new(0x33, 0x33, 0x33),
            cairo_context: cairo_context.clone(),
            canvas: Canvas::new(make_draw_handler(cairo_context), (500, 500)),
            draw_handler,
        }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        self.canvas.pen_stroke(input);
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
        self.size = (width, height)
    }
}
