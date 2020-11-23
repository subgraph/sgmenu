#![allow(deprecated)]
#[macro_use] 
extern crate glib;
extern crate gdk_pixbuf;
extern crate gdk;
extern crate gtk;
extern crate gtk_layer_shell_rs as gtk_layer_shell;
extern crate log;
extern crate env_logger;

use gio::prelude::*;
use std::env::args;

mod rowdata;
mod ui;

use ui::*;
fn main() {
    env_logger::init();
    let application =
        gtk::Application::new(Some("com.subgraph.sgmenu"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
       Ui::build_ui(app);
    });
    application.run(&args().collect::<Vec<_>>());
}
