mod canvas;

pub use canvas::PenInput;
use canvas::{Canvas, SingleVecImage, RGB};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

struct ViewportData {
    size: (usize, usize),
    background_color: RGB,
    cairo_context: Rc<RefCell<Option<cairo::Context>>>,
    canvas_display_matrix: cairo::Matrix,
}

#[derive(Debug)]
enum PenKind {
    PanCanvas,
    Circle,
    Zoom,
}

pub struct Viewport {
    data: Rc<RefCell<ViewportData>>,
    canvas: Canvas,
    draw_handler: Box<dyn Fn()>,
    pen_kind: PenKind,
    previous_input: Option<PenInput>,
    pressing_keys: HashSet<gdk::keys::Key>,
    stroke_start_position: Option<(f64, f64)>,
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
        context.set_matrix(viewport_data.canvas_display_matrix);
        context.set_source_surface(&image, 0.0, 0.0);
        context.get_source().set_filter(cairo::Filter::Nearest);
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
            background_color: RGB::new(0x33, 0x33, 0x40),
            cairo_context,
            canvas_display_matrix: cairo::Matrix::identity(),
        }));
        Viewport {
            data: data.clone(),
            canvas: Canvas::new(make_draw_handler(data), canvas_size),
            draw_handler,
            pen_kind: PenKind::Circle,
            previous_input: None,
            pressing_keys: HashSet::new(),
            stroke_start_position: None,
        }
    }

    fn apply_inv_matrix(&self, input: PenInput) -> PenInput {
        let PenInput { x, y, pressure } = input;
        let mut m = self.data.borrow().canvas_display_matrix.clone();
        m.invert();
        let (x, y) = m.transform_point(x, y);
        PenInput { x, y, pressure }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        let adjusted_input = self.apply_inv_matrix(input);
        match self.pen_kind {
            PenKind::Circle => {
                self.canvas.pen_stroke(adjusted_input);
            }
            PenKind::PanCanvas => {
                if let Some(i) = self.previous_input {
                    let i = &self.apply_inv_matrix(i);
                    let dx = adjusted_input.x - i.x;
                    let dy = adjusted_input.y - i.y;
                    self.move_canvas_relative(dx, dy);
                }
            }
            PenKind::Zoom => {
                let i = &self.previous_input;
                if let Some(previous) = i {
                    let ds = (2_f64).powf(
                        ((input.y - previous.y).powi(2)
                            + (input.x - previous.x).powi(2))
                        .sqrt()
                            * (previous.y - input.y).signum()
                            / 500.0,
                    );
                    self.zoom_canvas_relative(
                        ds,
                        if ds > 1.0 {
                            self.stroke_start_position.unwrap()
                        } else {
                            (adjusted_input.x, adjusted_input.y)
                        },
                    );
                }
            }
        }
        (self.draw_handler)();
        if self.previous_input.is_none() {
            self.stroke_start_position =
                Some((adjusted_input.x, adjusted_input.y));
        }
        self.previous_input = Some(input);
    }

    pub fn pen_stroke_end(&mut self) {
        self.previous_input = None;
        self.stroke_start_position = None;
        self.canvas.pen_stroke_end()
    }

    pub fn reflect_all(&mut self) {
        self.canvas.reflect_all();
        (self.draw_handler)();
    }

    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        {
            let mut data = self.data.borrow_mut();
            data.size = (width, height);
        }
        self.reflect_all()
    }

    fn move_canvas_relative(&mut self, dx: f64, dy: f64) {
        self.data
            .borrow_mut()
            .canvas_display_matrix
            .translate(dx, dy);
        self.reflect_all();
    }

    pub fn set_canvas_center(&mut self) {
        let size = self.data.borrow().size;
        let canvas_width = self.canvas.get_size().0 as f64;
        let canvas_height = self.canvas.get_size().1 as f64;
        self.move_canvas_relative(
            (size.0 as f64 - canvas_width) / 2.0,
            (size.1 as f64 - canvas_height) / 2.0,
        )
    }

    fn zoom_canvas_relative(&mut self, ds: f64, origin: (f64, f64)) {
        {
            let mut data = self.data.borrow_mut();
            data.canvas_display_matrix.scale(ds, ds);
            data.canvas_display_matrix
                .translate((1.0 - ds) * origin.0, (1.0 - ds) * origin.1);
        }
        self.reflect_all()
    }

    pub fn key_press(&mut self, key: gdk::keys::Key) {
        self.pressing_keys.insert(key);
        self.set_pen();
    }

    pub fn key_release(&mut self, key: gdk::keys::Key) {
        self.pressing_keys.remove(&key);
        self.set_pen();
    }

    fn set_pen(&mut self) {
        if self.pressing_keys
            == [gdk::keys::constants::space].iter().cloned().collect()
        {
            self.pen_kind = PenKind::PanCanvas
        } else if self.pressing_keys
            == [gdk::keys::constants::space, gdk::keys::constants::Control_L]
                .iter()
                .cloned()
                .collect()
        {
            self.pen_kind = PenKind::Zoom
        } else {
            self.pen_kind = PenKind::Circle
        }
    }
}
