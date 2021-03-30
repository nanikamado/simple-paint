use gtk::prelude::*;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

mod viewport;
use viewport::PenInput;

fn event_cb(
    position: (f64, f64),
    pressure: Option<f64>,
    event_type: gdk::EventType,
    mut viewport: RefMut<viewport::Viewport>,
) -> gtk::Inhibit {
    let (x, y) = position;
    use gdk::EventType::*;
    match event_type {
        ButtonPress | MotionNotify => viewport.pen_stroke(PenInput {
            x,
            y,
            pressure: pressure.unwrap_or(0.2),
        }),
        _ => (),
    };
    gtk::Inhibit(false)
}

fn make_connect_configure_event_cb(
    surface: Rc<RefCell<Option<cairo::Surface>>>,
    context: Rc<RefCell<Option<cairo::Context>>>,
    viewport: Rc<RefCell<viewport::Viewport>>,
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
        let width = width as usize;
        let height = height as usize;
        let mut viewport = viewport.borrow_mut();
        viewport.set_viewport_size(width, height);
        viewport.reflect_all();
        true
    }
}

pub fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Simple Paint");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1000, 800);

    let drawing = Rc::new(gtk::DrawingArea::new());

    drawing.add_events(gdk::EventMask::BUTTON1_MOTION_MASK);
    drawing.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing.add_events(gdk::EventMask::BUTTON_RELEASE_MASK);
    drawing.add_events(gdk::EventMask::KEY_PRESS_MASK);
    drawing.add_events(gdk::EventMask::KEY_RELEASE_MASK);

    let surface: Rc<RefCell<Option<cairo::Surface>>> =
        Rc::new(RefCell::new(None));
    let context: Rc<RefCell<Option<cairo::Context>>> =
        Rc::new(RefCell::new(None));
    let viewport = Rc::new(RefCell::new(viewport::Viewport::new(
        context.clone(),
        (0, 0),
        {
            let drawing_clone = drawing.clone();
            Box::new(move || drawing_clone.queue_draw())
        },
    )));

    drawing.connect_configure_event(make_connect_configure_event_cb(
        Rc::clone(&surface),
        context.clone(),
        viewport.clone(),
    ));

    let viewport_clone = viewport.clone();
    drawing.connect_button_press_event(move |_, e| {
        event_cb(
            e.get_position(),
            e.get_axis(gdk::AxisUse::Pressure),
            e.get_event_type(),
            viewport_clone.borrow_mut(),
        )
    });

    let viewport_clone = viewport.clone();
    drawing.connect_motion_notify_event(move |_, e| {
        event_cb(
            e.get_position(),
            e.get_axis(gdk::AxisUse::Pressure),
            e.get_event_type(),
            viewport_clone.borrow_mut(),
        )
    });

    drawing.connect_draw(move |_, c| {
        c.set_source_surface(surface.borrow().as_ref().unwrap(), 0.0, 0.0);
        c.paint();
        gtk::Inhibit(false)
    });

    let viewport_clone = viewport.clone();
    drawing.connect_button_release_event(move |_, _| {
        viewport_clone.borrow_mut().pen_stroke_end();
        gtk::Inhibit(false)
    });

    drawing.set_can_focus(true);
    let viewport_clone = viewport.clone();
    drawing.connect_key_press_event(move |_, key| {
        viewport_clone.borrow_mut().key_press(key.get_keyval());
        gtk::Inhibit(false)
    });

    let viewport_clone = viewport.clone();
    drawing.connect_key_release_event(move |_, key| {
        viewport_clone.borrow_mut().key_release(key.get_keyval());
        gtk::Inhibit(false)
    });

    drawing.connect_realize(move |_| {
        viewport.borrow_mut().set_canvas_center();
    });

    window.add(&*drawing);
    window.show_all();
}
