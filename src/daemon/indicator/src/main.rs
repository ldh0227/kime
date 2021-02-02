use gobject_sys::g_signal_connect_data;
use kimed_types::{deserialize_from, serialize_into, ClientHello, IndicatorMessage};
use libappindicator_sys::*;
use std::ffi::CString;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::ptr;

const HAN_ICON: &str = "kime-han-64x64.png";
const ENG_ICON: &str = "kime-eng-64x64.png";

macro_rules! cs {
    ($ex:expr) => {
        concat!($ex, "\0").as_ptr().cast()
    };
}

pub struct Indicator {
    indicator: *mut AppIndicator,
}

impl Indicator {
    pub fn new() -> Self {
        unsafe fn set_icon_path(indicator: *mut AppIndicator, path: &Path) {
            let s = path.to_str().unwrap();
            let s = CString::new(s).unwrap();
            libappindicator_sys::app_indicator_set_icon_theme_path(indicator, s.as_ptr());
        }

        unsafe {
            let m = gtk_sys::gtk_menu_new();
            let mi = gtk_sys::gtk_check_menu_item_new_with_label(cs!("Exit"));
            unsafe extern "C" fn exit() {
                gtk_sys::gtk_main_quit();
            }
            g_signal_connect_data(
                mi.cast(),
                cs!("activate"),
                Some(exit),
                ptr::null_mut(),
                None,
                0,
            );
            gtk_sys::gtk_menu_shell_append(m.cast(), mi.cast());
            let icon_dirs = xdg::BaseDirectories::with_profile("kime", "icons").unwrap();
            let indicator = libappindicator_sys::app_indicator_new(
                cs!("kime"),
                cs!(""),
                libappindicator_sys::AppIndicatorCategory_APP_INDICATOR_CATEGORY_APPLICATION_STATUS,
            );
            let han = icon_dirs.find_data_file(HAN_ICON).unwrap();
            let eng = icon_dirs.find_data_file(ENG_ICON).unwrap();
            set_icon_path(indicator, han.parent().unwrap());
            set_icon_path(indicator, eng.parent().unwrap());
            libappindicator_sys::app_indicator_set_status(
                indicator,
                AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE,
            );
            libappindicator_sys::app_indicator_set_menu(indicator, m.cast());
            gtk_sys::gtk_widget_show_all(m);
            Self { indicator }
        }
    }

    pub fn enable_hangul(&mut self) {
        unsafe {
            libappindicator_sys::app_indicator_set_icon_full(
                self.indicator,
                cs!("kime-han-64x64"),
                cs!("icon"),
            );
        }
    }

    pub fn disable_hangul(&mut self) {
        unsafe {
            libappindicator_sys::app_indicator_set_icon_full(
                self.indicator,
                cs!("kime-eng-64x64"),
                cs!("icon"),
            );
        }
    }
}

fn main() {
    unsafe {
        gtk_sys::gtk_init(ptr::null_mut(), ptr::null_mut());
    }

    let (indicator_tx, indicator_rx) =
        glib::MainContext::sync_channel(glib::PRIORITY_DEFAULT_IDLE, 10);

    std::thread::spawn(move || {
        let path = Path::new("/tmp/kimed.sock");

        while !path.exists() {
            std::thread::sleep(std::time::Duration::from_millis(600));
        }

        let conn = UnixStream::connect(path).unwrap();

        serialize_into(&conn, ClientHello::Indicator).unwrap();

        loop {
            let msg = deserialize_from(&conn).unwrap();

            match msg {
                IndicatorMessage::UpdateHangulState(state) => {
                    if indicator_tx.send(state).is_err() {
                        return;
                    }
                }
            }
        }
    });

    let ctx = glib::MainContext::ref_thread_default();
    assert!(ctx.acquire());

    let mut indicator = Indicator::new();

    indicator.disable_hangul();

    indicator_rx.attach(Some(&ctx), move |msg| {
        if msg {
            indicator.enable_hangul();
        } else {
            indicator.disable_hangul();
        }

        glib::Continue(true)
    });

    ctx.release();

    unsafe {
        gtk_sys::gtk_main();
    }
}
