use evdev::EventType;
use gtk::CssProvider;
use gtk::GestureClick;
use gtk::Label;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::gdk;
use gtk::prelude::*;
use gtk::style_context_add_provider_for_display;
use gtk::{Application, ApplicationWindow, glib};
use std::time::Duration;
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

async fn input_dispatch(label: &Label) {
    let kbd_path_dev = String::from("/dev/input/event4");
    let mut device = evdev::Device::open(std::env::args().nth(1).unwrap_or(kbd_path_dev)).unwrap();
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

                // TODO add met key handling
                // Inspect state
                //     if state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE) {
                //         print!("alt ");
                //     }

                if event.value() == KEY_STATE_PRESS {
                    let keysym = state.key_get_one_sym(keycode);
                    label.set_text(&keysym_get_name(keysym));
                    //println!("keysym: {} ", xkb::keysym_get_name(keysym));
                }
            }
        }
        glib::timeout_future(Duration::from_millis(10)).await;
    }
}
