use gtk::prelude::*;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

mod viewport;
use viewport::canvas::{Canvas, PenInput, SingleVecImage};

fn event_cb(
    position: (f64, f64),
    pressure: Option<f64>,
    event_type: gdk::EventType,
    mut backend: RefMut<Canvas>,
) -> gtk::Inhibit {
    let (x, y) = position;
    use gdk::EventType::*;
    match event_type {
        ButtonPress | MotionNotify => backend.pen_stroke(PenInput {
            x,
            y,
            pressure: pressure.unwrap_or(0.2),
        }),
        _ => (),
    };
    gtk::Inhibit(false)
}

fn make_drawer(
    widget: gtk::DrawingArea,
    context: Rc<RefCell<Option<cairo::Context>>>,
) -> Box<dyn Fn(&SingleVecImage, (usize, usize))> {
    Box::new(
        move |canvas: &SingleVecImage, canvas_size: (usize, usize)| {
            let stride = cairo::Format::Rgb24
                .stride_for_width(canvas.width as u32)
                .unwrap();
            let image = cairo::ImageSurface::create_for_data(
                canvas.vector.clone(),
                cairo::Format::Rgb24,
                canvas_size.0 as i32,
                canvas_size.1 as i32,
                stride,
            )
            .unwrap();
            let context = context.borrow();
            let context = context.as_ref().unwrap();
            context.set_source_surface(&image, 0.0, 0.0);
            context.paint();
            widget.queue_draw();
        },
    )
}

fn make_connect_configure_event_cb(
    surface: Rc<RefCell<Option<cairo::Surface>>>,
    backend: Rc<RefCell<Canvas>>,
    context: Rc<RefCell<Option<cairo::Context>>>,
) -> impl Fn(&gtk::DrawingArea, &gdk::EventConfigure) -> bool {
    move |w: &gtk::DrawingArea, _| {
        let width = w.get_allocated_width();
        let height = w.get_allocated_height();
        let s = w
            .get_window()
            .unwrap()
            .create_similar_surface(cairo::Content::Color, width, height)
            .unwrap();
        *context.borrow_mut() = Some(cairo::Context::new(&s));
        *surface.borrow_mut() = Some(s);
        let mut backend = backend.borrow_mut();
        let width = width as usize;
        let height = height as usize;
        backend.set_viewport_size(width, height);
        backend.reflect_all();
        true
    }
}

pub fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Simple Paint");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(500, 400);

    let drawing = gtk::DrawingArea::new();

    drawing.add_events(gdk::EventMask::BUTTON1_MOTION_MASK);
    drawing.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing.add_events(gdk::EventMask::BUTTON_RELEASE_MASK);
    let surface: Rc<RefCell<Option<cairo::Surface>>> =
        Rc::new(RefCell::new(None));
    let context: Rc<RefCell<Option<cairo::Context>>> =
        Rc::new(RefCell::new(None));

    let backend = Rc::new(RefCell::new(Canvas::new(
        make_drawer(drawing.clone(), Rc::clone(&context)),
        (0, 0),
    )));

    drawing.connect_configure_event(make_connect_configure_event_cb(
        Rc::clone(&surface),
        Rc::clone(&backend),
        context,
    ));

    let c_backend = Rc::clone(&backend);
    drawing.connect_button_press_event(move |_, e| {
        event_cb(
            e.get_position(),
            e.get_axis(gdk::AxisUse::Pressure),
            e.get_event_type(),
            c_backend.borrow_mut(),
        )
    });

    let c_backend = Rc::clone(&backend);
    drawing.connect_motion_notify_event(move |_, e| {
        event_cb(
            e.get_position(),
            e.get_axis(gdk::AxisUse::Pressure),
            e.get_event_type(),
            c_backend.borrow_mut(),
        )
    });

    drawing.connect_draw(move |_, c| {
        c.set_source_surface(surface.borrow().as_ref().unwrap(), 0.0, 0.0);
        c.paint();
        gtk::Inhibit(false)
    });

    drawing.connect_button_release_event(move |_, _| {
        backend.borrow_mut().pen_stroke_end();
        gtk::Inhibit(false)
    });

    window.add(&drawing);
    window.show_all();
}
