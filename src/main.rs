use std::fs;
use std::io;
use std::time::Duration;
use std::time::SystemTime;

use async_channel::Sender;
use evdev::EventType;
use glib::clone;
use gtk::CssProvider;
use gtk::GestureClick;
use gtk::Label;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::gdk;
use gtk::prelude::*;
use gtk::style_context_add_provider_for_display;
use gtk::{Application, ApplicationWindow, glib};
use xkbcommon::xkb;
use xkbcommon::xkb::Keycode;
use xkbcommon::xkb::keysym_get_name;

const APP_ID: &str = "org.gtk_rs.nenuphar";

static mut SHOW_BAR: bool = true;
const KEYCODE_OFFSET: u16 = 8;
const KEY_STATE_RELEASE: i32 = 0;
const KEY_STATE_PRESS: i32 = 1;
const KEY_STATE_REPEAT: i32 = 2;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let display = gdk::Display::default().expect("No display found");
    let label = Label::builder().label("click to hide title bar").build();
    label.add_css_class("main-label");
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&label)
        .title("nenuphar")
        .build();

    let css = "
        window {
            background-color: transparent;
        }
        .main-label {
            background-color: transparent;
            font-size: 54pt;
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

    let (sender, receiver) = async_channel::unbounded();

    for kb_evt_path in get_kbd_dev_event_paths().unwrap() {
        let sender_clone = sender.clone();
        std::thread::spawn(move || {
            let future = input_dispatch(sender_clone, kb_evt_path);
            futures::executor::block_on(future);
        });
    }

    glib::spawn_future_local(clone!(
        #[weak]
        label,
        async move {
            while let Ok(msg) = receiver.recv().await {
                label.set_text(&msg);
            }
        }
    ));
}

pub fn get_kbd_dev_event_paths() -> io::Result<Vec<String>> {
    let content = fs::read_to_string("/proc/bus/input/devices")?;
    let events = content
        .lines()
        .filter(|line| line.starts_with('H') && line.contains("kbd") && line.contains("leds"))
        .flat_map(|line| {
            line.split_whitespace()
                .filter(|event| event.starts_with("event"))
                .map(|event| format!("/dev/input/{event}"))
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(events)
}

async fn input_dispatch(sender: Sender<String>, kbd_evt_path: String) {
    let mut device = evdev::Device::open(kbd_evt_path).unwrap();
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

    let keymap = xkb::Keymap::new_from_names(
        &context,
        "evdev",                                     // rules
        "pc105",                                     // model
        "pl",                                        // layout
        "",                                          // variant
        Some("terminate:ctrl_alt_bksp".to_string()), // options
        xkb::COMPILE_NO_FLAGS,
    )
    .unwrap();

    let mut state = xkb::State::new(&keymap);
    let mut last_ts = SystemTime::now();
    let mut last_res = String::new();
    let mut res = String::new();
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;

    loop {
        for event in device.fetch_events().unwrap() {
            if event.event_type() == EventType::KEY {
                let keycode: Keycode = (event.code() + KEYCODE_OFFSET).into();

                if event.value() == KEY_STATE_REPEAT && !keymap.key_repeats(keycode) {
                    continue;
                }

                if event.value() == KEY_STATE_RELEASE {
                    state.update_key(keycode, xkb::KeyDirection::Up)
                } else {
                    state.update_key(keycode, xkb::KeyDirection::Down)
                };

                if event.value() == KEY_STATE_PRESS {
                    let mut prefix = String::from("");
                    ctrl = state.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
                    alt = state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
                    let keysym = state.key_get_one_sym(keycode);
                    //xkb::Keysym::Kana_Shift

                    if ctrl {
                        prefix.push_str("ctrl + ");
                    }
                    alt = state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
                    if alt {
                        prefix.push_str("alt + ");
                    }

                    let mut key = keysym_get_name(keysym);
                    if keysym.is_modifier_key() {
                        //                        key = String::from("");
                    }

                    let ts = event.timestamp().duration_since(last_ts).unwrap();
                    println!("{ts:?}");

                    if ts > Duration::from_millis(2000) || ctrl || alt {
                        last_res = String::new();
                    }

                    res = format!("{last_res}{prefix}{key}");
                    last_ts = event.timestamp();

                    last_res = format!("{res} ");
                    sender.send(res).await.unwrap();
                }
            }
        }
    }
}
