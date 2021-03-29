use gio::prelude::*;

mod ui;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.nanikamado.simplepaint"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        ui::build_ui(app);
    });

    application.run(&std::env::args().collect::<Vec<_>>());
}
