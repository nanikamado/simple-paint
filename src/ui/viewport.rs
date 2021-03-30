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
enum PenKind {
    PanCanvas,
    Circle,
}

pub struct Viewport {
    data: Rc<RefCell<ViewportData>>,
    canvas: Canvas,
    draw_handler: Box<dyn Fn()>,
    pen_kind: PenKind,
    previous_input: Option<PenInput>,
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
        let canvas_size = (500, 500);
        let data = Rc::new(RefCell::new(ViewportData {
            size,
            background_color: RGB::new(0x33, 0x33, 0x33),
            cairo_context: cairo_context.clone(),
            canvas_position: (
                (size.0 as f64 - canvas_size.0 as f64) / 2.0,
                (size.0 as f64 - canvas_size.1 as f64) / 2.0,
            ),
        }));
        Viewport {
            data: data.clone(),
            canvas: Canvas::new(make_draw_handler(data), canvas_size),
            draw_handler,
            pen_kind: PenKind::Circle,
            previous_input: None,
        }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        let PenInput { x, y, pressure } = input;
        let canvas_position = self.data.borrow().canvas_position;
        match self.pen_kind {
            PenKind::Circle => {
                let x = x - canvas_position.0;
                let y = y - canvas_position.1;
                self.canvas.pen_stroke(PenInput { x, y, pressure });
            }
            PenKind::PanCanvas => {
                let i = &self.previous_input;
                if let Some(i) = i {
                    let dx = x - i.x;
                    let dy = y - i.y;
                    self.move_canvas_relative(dx, dy);
                }
            }
        }
        (self.draw_handler)();
        self.previous_input = Some(input);
    }

    pub fn pen_stroke_end(&mut self) {
        self.previous_input = None;
        self.canvas.pen_stroke_end()
    }

    pub fn reflect_all(&mut self) {
        self.canvas.reflect_all();
        (self.draw_handler)();
    }

    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        let mut data = self.data.borrow_mut();
        data.size = (width, height);
    }

    fn move_canvas_relative(&mut self, dx: f64, dy: f64) {
        let (x, y) = self.data.borrow().canvas_position;
        self.move_canvas(x + dx, y + dy);
    }

    fn move_canvas(&mut self, x: f64, y: f64) {
        {
            let mut data = self.data.borrow_mut();
            data.canvas_position = (x, y);
        }
        self.canvas.reflect_all();
        (self.draw_handler)();
    }

    pub fn set_canvas_center(&mut self) {
        let size = self.data.borrow().size;
        let canvas_width = self.canvas.get_size().0 as f64;
        let canvas_height = self.canvas.get_size().1 as f64;
        self.move_canvas(
            (size.0 as f64 - canvas_width) / 2.0,
            (size.1 as f64 - canvas_height) / 2.0,
        )
    }

    pub fn key_press(&mut self, key: gdk::keys::Key) {
        match key {
            gdk::keys::constants::space => self.pen_kind = PenKind::PanCanvas,
            _ => {}
        }
    }

    pub fn key_release(&mut self, key: gdk::keys::Key) {
        match key {
            gdk::keys::constants::space => self.pen_kind = PenKind::Circle,
            _ => {}
        }
    }
}
