use std::fs::File;
use std::fs::OpenOptions;
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use gtk::CssProvider;
use gtk::GestureClick;
use gtk::Label;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::gdk;
use gtk::prelude::*;
use gtk::style_context_add_provider_for_display;
use gtk::{Application, ApplicationWindow, glib};
use input::Event;
use input::Libinput;
use input::LibinputInterface;
use input::event::keyboard::KeyboardEventTrait;
use libc::O_RDONLY;
use libc::O_RDWR;
use libc::O_WRONLY;

const APP_ID: &str = "org.gtk_rs.nenuphar";

static mut SHOW_BAR: bool = true;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let display = gdk::Display::default().expect("No display found");
    let label = Label::builder().label("this is me").build();
    label.add_css_class("main-label");

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&label)
        .build();

    let css = "
        window {
            background-color: transparent;
        }
        .main-label {
            background-color: transparent;
            font-size: 42pt;
            color: white;
        }
    ";
    let provider = CssProvider::new();
    provider.load_from_string(css);

    style_context_add_provider_for_display(
        &display,
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let win_ref = window.clone();

    let gesture = GestureClick::new();
    gesture.connect_pressed(move |_gesture, _n_press, _x, _y| {
        unsafe {
            SHOW_BAR = !SHOW_BAR;
            win_ref.set_decorated(SHOW_BAR);
        };
    });

    label.add_controller(gesture);
    window.present();
}

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file: File| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn update_key_events() {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();
    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(k) => {
                    println!("{:?} {}", k.key_state(), k.key())
                }
                _ => println!("Got event: {:?}", event),
            }
        }
    }
}
