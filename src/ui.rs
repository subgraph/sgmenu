use gio::prelude::*;
use gtk::prelude::*;
use std::{thread, time};
use log::{info, warn};
use gtk::SettingsExt;
use gtk::StyleContext;
use crate::rowdata::row_data::RowData;
use crate::rowdata::row_data::RowDataExt;
pub enum Msg {
    LaunchApplication(String),                                                                                                                                                        
    Quit
}
#[derive(Clone)]
pub struct Ui {
    entry: gtk::Entry,
    listbox: gtk::ListBox,
    model: gio::ListStore,
    sender: glib::Sender<Msg>
}
const STYLE: &str = include_str!("../data/style.css");
const APP_EXCEPTIONS: &'static [&'static str] = &["org.freedesktop.IBus.Setup.desktop", "org.gnome.Screenshot.desktop"];

impl Ui {

    pub fn build_ui(application: &gtk::Application) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);                                                                                                               
        let listbox = gtk::ListBox::new();
        let window = Self::build_window(&application, &listbox, &sender);
        let model = Self::build_model();
        let entry = Self::build_entry(&sender, &listbox, &model);
        listbox.set_selection_mode(gtk::SelectionMode::Single);
        listbox.emit_move_cursor(gtk::MovementStep::DisplayLines, 1);
        let scrolled_window = Self::build_scrolled_window();
        scrolled_window.add(&listbox);
        listbox.set_can_focus(false);
        let ui = Self {
            entry,
            listbox,
            model,
            sender
        };
        ui.setup_style();
        ui.bind_model();
        ui.setup_filter();
        ui.setup_row_activated();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&ui.entry, false, false, 0);
        scrolled_window.set_can_focus(false);
        vbox.pack_start(&scrolled_window, true, true, 0);
        vbox.set_vexpand(true);
        vbox.set_hexpand(true); 
        window.add(&vbox);
        receiver.attach(None,clone!(@strong application, @strong window =>  move |msg| { 
            match msg {
                Msg::LaunchApplication(app_id) => {
                        let result = Self::launch_application(&app_id);
                        match result {
                            Ok(_) => { 
                                // Delay is for some naughty apps such as Nautilus
                                let delay = time::Duration::from_millis(500);
                                window.hide();
                                gdk::Window::process_all_updates();
                                thread::sleep(delay);
                                application.quit();
                            },
                            Err(e) => {
                                warn!("Could not launch application {}: {}", app_id, e);
                            }
                        }
                },
                Msg::Quit => {
                    application.quit();
                }
            }
            glib::Continue(true)
        }));
        ui.populate_model();
        ui.setup_entry_changed();
        ui.select_first_row();
        window.show_all();
        ui
    }
    fn is_app_exception(app: &str) -> bool {
        let result = APP_EXCEPTIONS.to_vec().into_iter().find(|name|
            *name == app
        );
        result.is_some()
    }

    fn populate_model(&self) {
        info!("Getting list of applications");
        let app_infos = gio::AppInfo::get_all();
        for app in app_infos {
            if let Some(app_id) = app.get_id() {
                if app.should_show() && !Self::is_app_exception(&app_id) {
                    if let Some(row) = Self::parse_app_info(&app_id, &app) {
                        self.model.append(&row);
                    }
                }
            }
        }
    }

    fn parse_app_info(app_id: &str, app_info: &gio::AppInfo) -> Option<RowData> {
        if let Some(desktop_app_info) = gio::DesktopAppInfo::new(&app_id) { 
            let mut display_name = String::new();
            if let Some(name) = app_info.get_display_name() {
                display_name = name.to_string();
            }
            let mut icon = String::new();
            if let Some(icon_name) = app_info.get_icon() {
                if let Some(icon_ext) = gio::IconExt::to_string(&icon_name) {
                    icon = icon_ext.to_string();
                } 
            }
            let mut categories = String::new();
            if let Some(app_categories) = desktop_app_info.get_categories() {
                categories = app_categories.to_string();
            }
            let mut keywords = String::new();
            if let Some(app_keywords) = desktop_app_info.get_categories() {
                keywords = app_keywords.to_string();
            }
            let row = RowData::new(
                &app_id,
                &display_name,
                &icon,
                &categories,
                &keywords
            );
            return Some(row)
        }
        None 
    }

    fn launch_application(app_id: &str) -> Result<(), glib::error::Error> {
        if let Some(app_info) = gio::DesktopAppInfo::new(app_id) {
            info!("Launching application: {}", app_id);
            let result = app_info.launch_uris_as_manager(
                &[],
                None::<&gio::AppLaunchContext>,
                glib::SpawnFlags::SEARCH_PATH | glib::SpawnFlags::DO_NOT_REAP_CHILD | glib::SpawnFlags::LEAVE_DESCRIPTORS_OPEN,
                None,
                None
            );
            return result
                    
        }
        Err(glib::error::Error::new(glib::KeyFileError::NotFound, "Could not find the .desktop file"))
    }

    fn select_first_row(&self) {
        let n_items = self.model.get_n_items();
        self.listbox.unselect_all();
        for i in 0..n_items {
            let row = self.listbox.get_row_at_index(i as i32).unwrap();
            if row.get_visible() && row.get_child_visible() {
                self.listbox.select_row(Some(&row));
                return
            }
        }
    }

    fn build_window(application: &gtk::Application, listbox: &gtk::ListBox, sender: &glib::Sender<Msg>) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);
        Self::setup_layer(&window);
        window.set_opacity(0.90);
        window.set_property_default_height(800);
        window.set_property_default_width(400);
        window.connect_delete_event(clone!(@strong application => move |_, _| {
            application.quit();
            Inhibit(false)
        }));
        window.connect_key_press_event(clone!(@strong sender, @strong listbox => move |_, key| {
            match key.get_keyval() {
                gdk::keys::constants::Escape => {
                    let _ = sender.send(Msg::Quit);
                },
                gdk::keys::constants::Up => {
                    if let Some(selected) = listbox.get_selected_rows().first() {
                        let index = selected.get_index() as u32;
                        for i in (0..index).rev() {
                            if let Some(row) = listbox.get_row_at_index(i as i32) {
                                if row.get_visible() && row.get_child_visible() {
                                    listbox.select_row(Some(&row));
                                    break;
                                }
                            }
                        }
                    }
                },
                gdk::keys::constants::Down => {
                    if let Some(selected) = listbox.get_selected_rows().first() {
                        let index = selected.get_index() as u32;
                        let n_items = listbox.get_children().len() as u32;
                        for i in index+1..n_items {
                            if let Some(row) = listbox.get_row_at_index(i as i32) {
                                if row.get_visible() && row.get_child_visible() {
                                    listbox.select_row(Some(&row));
                                    break;
                                }
                            }
                        }
                    }
                    
                },
                gdk::keys::constants::Super_L | gdk::keys::constants::Super_R => {
                    info!("Super key is disabled");
                    return gtk::Inhibit(true);
                },
                _ => ()
            }
            gtk::Inhibit(false)
        }));
        window
    } 

    fn build_scrolled_window() -> gtk::ScrolledWindow {
        let scrolled_window = gtk::ScrolledWindow::new(Option::<&gtk::Adjustment>::None, Option::<&gtk::Adjustment>::None);
        scrolled_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled_window
    }

    fn build_entry(sender: &glib::Sender<Msg>, listbox: &gtk::ListBox, model: &gio::ListStore) -> gtk::Entry {
        let entry = gtk::Entry::new();
        entry.connect_activate(clone!(@strong sender, @strong listbox, @strong model => move |_entry| {
            if let Some(row) = listbox.get_selected_rows().first() {
                // TODO: Turn this into a function as it is also the same code on the row activation
                let index = row.get_index();
                let item = model.get_object(index as u32).unwrap();
                let data = item.downcast_ref::<RowData>().expect("Row data is of wrong type");
                let app_id = data.get_app_id();
                let _ = sender.send(Msg::LaunchApplication(app_id));
            }
        }));
        entry
    }

    fn setup_filter(&self) {
        let entry = self.entry.clone();
        let model = self.model.clone();
        let listbox = self.listbox.clone();
        listbox.set_filter_func(Some(Box::new(clone!(@strong entry, @strong model, @strong listbox => move |row| {
            let index = row.get_index();
            if let Some(item) = model.get_object(index as u32) {
                let data = item.downcast_ref::<RowData>().expect("Row data is of wrong type");
                let query = entry.get_text();
                return data.match_any(&query)
            }
            false
        }))));
    }

    fn setup_layer(window: &gtk::ApplicationWindow) {
        let window = window.clone().upcast::<gtk::Window>();
        gtk_layer_shell::init_for_window(&window);
        gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
        gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 200); 
        gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Bottom, 200); 
        gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 200); 
        gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 200);
        gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
        gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
        gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
        gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);
        gtk_layer_shell::set_exclusive_zone(&window, -1);
        gtk_layer_shell::set_keyboard_interactivity(&window, true);
    }

    fn setup_style(&self) {
        if let Some(settings) = gtk::Settings::get_default() {
            settings.set_property_gtk_application_prefer_dark_theme(true);
        }
        let css = gtk::CssProvider::new();

        if let Err(err) = css.load_from_data(STYLE.as_bytes()) {
            println!("Error parsing CSS style: {}", err);
            return;
        }
        if let Some(screen) = gdk::Screen::get_default() {
            StyleContext::add_provider_for_screen(&screen, &css, gtk::STYLE_PROVIDER_PRIORITY_USER);
        }
    }

    fn build_model() -> gio::ListStore {
        let model = gio::ListStore::new(RowData::static_type());
        model
    }


    fn bind_model(&self) {
        let ui = self.clone();
        let entry = ui.entry.clone();
        let listbox= ui.listbox.clone();
        let model = ui.model.clone();
        listbox.bind_model(Some(&model), clone!(@strong entry, @strong model => move |item| {
            let row = gtk::ListBoxRow::new();
            let item = item.downcast_ref::<RowData>().expect("Row data is of wrong type");
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            let icon = gtk::Image::new();
            icon.set_pixel_size(24);
            item.bind_property("icon_name", &icon, "icon-name")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
            let display_name_label = gtk::Label::new(Some("")); 
            display_name_label.set_halign(gtk::Align::Start);
            item.bind_property("display_name", &display_name_label, "label")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
            hbox.pack_start(&icon, false, false, 0);
            hbox.pack_start(&display_name_label, true, true, 0);
            hbox.set_can_focus(false);
            row.add(&hbox);
            row.set_can_focus(false); 
            row.show_all();
            row.upcast::<gtk::Widget>()
        }));
    }

    fn setup_row_activated(&self) {
        let ui = self.clone();
        let listbox = ui.listbox.clone();
        let model = ui.model.clone();
        let sender = ui.sender.clone();
        listbox.connect_row_activated(clone!(@strong model, @strong sender => move |_listbox, row| {
            let index = row.get_index();
            if let Some(item) = model.get_object(index as u32) {
                let data = item.downcast_ref::<RowData>().expect("Row data is of wrong type");
                let app_id = data.get_app_id();
                let _ = sender.send(Msg::LaunchApplication(app_id));
            }
        }));
    }

    fn setup_entry_changed(&self) {
        let ui = self.clone();
        let entry = ui.entry.clone();
        let listbox = ui.listbox.clone();
        entry.connect_changed(clone!(@strong listbox, @strong ui => move |_entry| {
            listbox.invalidate_filter();
            ui.select_first_row();
        }));
    }

}