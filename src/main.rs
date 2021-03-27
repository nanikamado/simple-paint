use gio::prelude::*;

mod build_ui;
mod backend;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.nanikamado.simplepaint"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui::build_ui(app);
    });

    application.run(&std::env::args().collect::<Vec<_>>());
}
