use gtk3::{prelude::*, Menu, MenuItem, glib};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use zbus::{SignalContext, blocking::{Connection}};
use std::{sync::Mutex};

lazy_static::lazy_static! {
    static ref CLOSE_REQUEST: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_STATUS: Mutex<String> = Mutex::new(String::new());
}

struct TrayIcon;

#[zbus::dbus_interface(name = "com.hunterwittenborn.CelesteTray")]
impl TrayIcon {
    async fn close(&self) {
        *(*CLOSE_REQUEST).lock().unwrap() = true;
    }

    async fn update_status(&self, status: &str) {
        *(*CURRENT_STATUS).lock().unwrap() = status.to_string();
    }
}

fn main() {
    gtk3::init().unwrap();
    
    // The indicator.
    let mut indicator = AppIndicator::new("Celeste", "com.hunterwittenborn.CelesteTray-symbolics.svg");
    indicator.set_status(AppIndicatorStatus::Active);

    let mut menu = Menu::new();
    let menu_sync_status = MenuItem::builder().sensitive(false).build();
    let menu_quit = MenuItem::builder().label("Quit").build();
    menu.append(&menu_sync_status);
    menu.append(&menu_quit);
    indicator.set_menu(&mut menu);

    // Our DBus connection to receive messages from the main application.
    let connection = Connection::session().unwrap();
    connection.object_server().at(libceleste::DBUS_TRAY_OBJECT, TrayIcon).unwrap();
    connection.request_name(libceleste::TRAY_ID).unwrap();
    
    menu_quit.connect_activate(|menu_quit| {
        *(*CLOSE_REQUEST).lock().unwrap() = true;
    });
    menu.show_all();
    
    let send_close = || connection.call_method(
        Some(libceleste::DBUS_APP_ID),
        libceleste::DBUS_APP_OBJECT,
        Some(libceleste::DBUS_APP_ID),
        "Close",
        &()
    );

    loop {
        gtk3::main_iteration().then_some(()).unwrap();
        let status = (*(*CURRENT_STATUS).lock().unwrap()).clone();
        indicator.set_title(&status);
        menu_sync_status.set_label(&status);

        if *(*CLOSE_REQUEST).lock().unwrap() {
            // Set up the quit label.
            menu_quit.set_sensitive(false);
            menu_quit.set_label("Quitting...");

            // Notify the tray icon to close.
            // I'm not sure when this can fail, so output an error if one is received.
            if let Err(err) = connection.call_method(
                Some(libceleste::DBUS_APP_ID),
                libceleste::DBUS_APP_OBJECT,
                Some(libceleste::DBUS_APP_ID),
                "Close",
                &()
            ) {
                hw_msg::warningln!("Got error while sending close request to tray: '{err}'.");
            };
    
            // And then quit the application.
            break
        }
    }
}
