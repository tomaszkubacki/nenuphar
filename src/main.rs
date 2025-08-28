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
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::Duration;

use libc::O_RDWR;
use libc::O_WRONLY;

const APP_ID: &str = "org.gtk_rs.nenuphar";

static mut SHOW_BAR: bool = true;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    let exit_code = app.run();
    print!("the end");
    exit_code
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
            font-size: 44pt;
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
        //        label_clone.lock().unwrap().set_text("ooo!");
        unsafe {
            SHOW_BAR = !SHOW_BAR;
            win_ref.set_decorated(SHOW_BAR);
        };
    });

    label.add_controller(gesture);
    window.present();
    glib::spawn_future_local(async move {
        input_dispatch(&label).await;
    });
}

#[derive(Debug)]
pub struct Keys {
    shift: bool,
    alt: bool,
    meta: bool,
    key: u32,
}

async fn input_dispatch(label: &Label) {
    let mut input = Libinput::new_with_udev(Interface);
    let mut keys = Keys {
        shift: false,
        alt: false,
        meta: false,
        key: 0,
    };
    let key_map = key_map();
    let mut display: &'static str;
    input.udev_assign_seat("seat0").unwrap();
    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            if let Event::Keyboard(k) = event {
                keys.key = k.key();
                if key_map.contains_key(&k.key()) {
                    display = key_map.get(&k.key()).unwrap()
                } else {
                    display = "";
                    println!("{}", &k.key());
                }

                label.set_text(display);
            }
        }
        glib::timeout_future(Duration::from_millis(10)).await;
    }
}

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read(true)
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file: File| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn key_map() -> HashMap<u32, &'static str> {
    HashMap::from([(1, "ESC"), (31, "s"), (30, "a")])
}
