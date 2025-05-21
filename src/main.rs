use gtk;
use std::{path::Path, thread, time::Duration, sync::mpsc};
use tray_icon::{TrayIconBuilder, Icon, menu::{Menu, MenuItem}};
use image::io::Reader as ImageReader;

// Check if the device is enabled
fn poll_device(device_path: &str) -> bool {
    Path::new(device_path).exists()
}

fn load_icon(path: &str) -> Icon {
    let image = ImageReader::open(path).unwrap().decode().unwrap().into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).unwrap()
}

fn main() {
    // Touchscreen path
    let device_path = "/sys/bus/i2c/drivers/i2c_hid_acpi/i2c-GXTP7386:00";
    let enabled_path = home::home_dir().map(|p| p.join(".local/share/touchscreen-status-monitor/icons/enabled.png")).expect("Failed to get home directory").to_string_lossy().into_owned();
    let disabled_path = home::home_dir().map(|p| p.join(".local/share/touchscreen-status-monitor/icons/disabled.png")).expect("Failed to get home directory").to_string_lossy().into_owned();
    println!("{}\n{}", enabled_path, disabled_path);
    gtk::init().unwrap();

    // Create tray menu
    let trayMenu = Menu::new();
    // This button does literally nothing
    let quitButton = MenuItem::new("Quit", true, None);
    trayMenu.append(&quitButton).unwrap();

    // Create tray icon
    let trayIcon = TrayIconBuilder::new()
        .with_tooltip("Touchscreen Status")
        .with_icon(load_icon(&enabled_path))
        .with_menu(Box::new(trayMenu))
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut last = None;
        loop {
            let status = poll_device(device_path);
            if Some(status) != last {
                last = Some(status);
                let iconPath = if status {&enabled_path} else {&disabled_path};
                tx.send(iconPath.to_string()).ok();
            }
            thread::sleep(Duration::from_secs(5));
        }
    });

    glib::source::timeout_add_local(Duration::from_millis(1000), move || {
        if let Ok(iconPath) = rx.try_recv() {
            let icon = load_icon(&iconPath);
            trayIcon.set_icon(Some(icon)).unwrap();
        }
        glib::ControlFlow::Continue
    });
    gtk::main();
}
