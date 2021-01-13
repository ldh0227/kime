use gdk_sys::{
    gdk_keyval_to_unicode, GdkEventKey, GdkWindow, GDK_CONTROL_MASK, GDK_KEY_PRESS, GDK_MOD1_MASK,
    GDK_MOD2_MASK, GDK_MOD3_MASK, GDK_MOD4_MASK, GDK_MOD5_MASK, GDK_SHIFT_MASK,
};
use glib_sys::{g_strcmp0, gboolean, gpointer, GType, GFALSE, GTRUE};
use gobject_sys::{
    g_object_new, g_object_ref, g_object_unref, g_signal_emit, g_signal_lookup,
    g_type_check_class_cast, g_type_check_instance_cast, g_type_module_register_type,
    g_type_module_use, g_type_register_static, GObject, GObjectClass, GTypeClass, GTypeInfo,
    GTypeInstance, GTypeModule, G_TYPE_OBJECT,
};
use gtk_sys::{gtk_im_context_get_type, GtkIMContext, GtkIMContextClass, GtkIMContextInfo};
use once_cell::sync::{Lazy, OnceCell};
use std::mem::{size_of, MaybeUninit};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr::{self, NonNull};

use kime_engine::{Config, InputEngine, InputResult};

#[repr(transparent)]
struct TypeInfoWrapper(GTypeInfo);

unsafe impl Send for TypeInfoWrapper {}
unsafe impl Sync for TypeInfoWrapper {}

#[repr(transparent)]
struct ContextInfoWrapper(GtkIMContextInfo);

unsafe impl Send for ContextInfoWrapper {}
unsafe impl Sync for ContextInfoWrapper {}

macro_rules! cs {
    ($text:expr) => {
        concat!($text, "\0").as_ptr().cast::<c_char>()
    };
}

struct KimeIMSignals {
    commit: c_uint,
    delete_surrounding: c_uint,
}

impl KimeIMSignals {
    pub unsafe fn new(class: *mut KimeIMContextClass) -> Self {
        let ty = type_of_class(class.cast());

        macro_rules! sig {
            ($($name:ident),+) => {
                $(
                    let $name = g_signal_lookup(cs!(stringify!($name)), ty);
                    assert_ne!($name, 0);
                )+
                return Self { $($name,)+ };
            }
        }

        sig!(commit, delete_surrounding);
    }
}

static SIGNALS: OnceCell<KimeIMSignals> = OnceCell::new();
static CONFIG: Lazy<Config> = Lazy::new(|| Config::load_from_config_dir().unwrap_or_default());

#[repr(C)]
struct KimeIMContextClass {
    _parent: GtkIMContextClass,
}

#[repr(C)]
struct KimeIMContext {
    parent: GtkIMContext,
    client_window: Option<NonNull<GdkWindow>>,
    engine: InputEngine,
    preedit_state: bool,
}

impl KimeIMContext {
    pub fn as_obj(&mut self) -> *mut GObject {
        &mut self.parent.parent_instance
    }

    pub fn filter_keypress(&mut self, key: &GdkEventKey) -> bool {
        let code = match kime_engine::KeyCode::from_hardward_code(key.hardware_keycode) {
            Some(code) => code,
            None => {
                return false;
            }
        };

        let ret = self.engine.press_key(
            kime_engine::Key::new(code, key.state & GDK_SHIFT_MASK != 0),
            &CONFIG,
        );

        dbg!(ret);

        match ret {
            InputResult::Commit(c) => {
                self.clear_preedit();
                self.commit(c);
                true
            }
            InputResult::CommitCommit(f, s) => {
                self.clear_preedit();
                self.commit(f);
                self.commit(s);
                true
            }
            InputResult::CommitBypass(c) => {
                self.clear_preedit();
                self.commit(c);
                false
            }
            InputResult::CommitPreedit(c, p) => {
                self.clear_preedit();
                self.commit(c);
                self.preedit(p);
                true
            }
            InputResult::Preedit(p) => {
                self.clear_preedit();
                self.preedit(p);
                true
            }
            InputResult::ClearPreedit => false,
            InputResult::Bypass => false,
            InputResult::Consume => true,
        }
    }

    pub fn reset(&mut self) {
        self.preedit_state = false;
    }

    pub fn preedit(&mut self, c: char) {
        eprintln!("preedit: {}", c);
        self.commit(c);
        self.preedit_state = true;
    }

    pub fn clear_preedit(&mut self) {
        if self.preedit_state {
            self.preedit_state = false;
            self.delete_surronding(-1, 1);
        }
    }

    pub fn delete_surronding(&mut self, offset: c_int, count: c_uint) {
        eprintln!("delete");
        unsafe {
            let mut return_value = MaybeUninit::<gboolean>::uninit();
            g_signal_emit(
                self.as_obj(),
                SIGNALS.get().unwrap().delete_surrounding,
                0,
                offset,
                count,
                return_value.as_mut_ptr(),
            );

            eprintln!("ret: {}", return_value.assume_init());
        }
    }

    pub fn commit(&mut self, c: char) {
        eprintln!("commit: {}", c);

        let mut buf = [0; 8];
        c.encode_utf8(&mut buf);
        unsafe {
            g_signal_emit(
                self.as_obj(),
                SIGNALS.get().unwrap().commit,
                0,
                buf.as_ptr(),
            );
        }
    }
}

static KIME_TYPE_IM_CONTEXT: OnceCell<GType> = OnceCell::new();

unsafe fn type_of_class(class: *mut GTypeClass) -> GType {
    (*class).g_type
}

unsafe fn register_module(module: *mut GTypeModule) {
    unsafe extern "C" fn im_context_class_init(class: gpointer, _data: gpointer) {
        let class = class.cast::<KimeIMContextClass>();

        let im_context_class = g_type_check_class_cast(class.cast(), gtk_im_context_get_type())
            .cast::<GtkIMContextClass>()
            .as_mut()
            .unwrap();
        let gobject_class =
            g_type_check_class_cast(class.cast(), G_TYPE_OBJECT).cast::<GObjectClass>();

        im_context_class.set_client_window = Some(set_client_window);
        im_context_class.filter_keypress = Some(filter_keypress);
        im_context_class.reset = Some(reset_im);
        im_context_class.focus_in = Some(focus_in);
        im_context_class.focus_out = Some(focus_out);
        im_context_class.set_cursor_location = None;
        im_context_class.set_use_preedit = None;

        SIGNALS.get_or_init(|| KimeIMSignals::new(class));

        (*gobject_class).finalize = Some(im_context_instance_finalize);
    }

    unsafe extern "C" fn focus_in(_ctx: *mut GtkIMContext) {}

    unsafe extern "C" fn focus_out(ctx: *mut GtkIMContext) {
        reset_im(ctx);
    }

    unsafe extern "C" fn reset_im(ctx: *mut GtkIMContext) {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        ctx.reset();
    }

    unsafe extern "C" fn filter_keypress(
        ctx: *mut GtkIMContext,
        key: *mut GdkEventKey,
    ) -> gboolean {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        let key = key.as_mut().unwrap();

        let skip_mask = GDK_CONTROL_MASK
            | GDK_MOD1_MASK
            | GDK_MOD2_MASK
            | GDK_MOD3_MASK
            | GDK_MOD4_MASK
            | GDK_MOD5_MASK;

        if key.type_ != GDK_KEY_PRESS {
            GFALSE
        // skip modifiers
        } else if key.state & skip_mask != 0 {
            ctx.reset();
            GFALSE
        } else if ctx.filter_keypress(key) {
            GTRUE
        } else {
            ctx.reset();

            if CONFIG.gtk_commit_english {
                let c = std::char::from_u32_unchecked(gdk_keyval_to_unicode(key.keyval));

                if !c.is_control() {
                    ctx.commit(c);
                    return GTRUE;
                }
            }

            GFALSE
        }
    }

    unsafe extern "C" fn set_client_window(ctx: *mut GtkIMContext, window: *mut GdkWindow) {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        let window = NonNull::new(window);

        if let Some(prev_win) = ctx.client_window {
            g_object_unref(prev_win.as_ptr().cast());
        }

        if let Some(win) = window {
            g_object_ref(win.as_ptr().cast());
        }

        ctx.client_window = window;
    }

    unsafe extern "C" fn im_context_class_finalize(class: gpointer, _data: gpointer) {
        let _class = class.cast::<KimeIMContextClass>();
    }

    unsafe extern "C" fn im_context_instance_init(ctx: *mut GTypeInstance, _class: gpointer) {
        let parent = ctx.cast::<GtkIMContext>();

        ctx.cast::<KimeIMContext>().write(KimeIMContext {
            parent: parent.read(),
            client_window: None,
            engine: InputEngine::new(),
            preedit_state: false,
        });
    }

    unsafe extern "C" fn im_context_instance_finalize(ctx: *mut GObject) {
        let ctx = ctx.cast::<KimeIMContext>();
        ctx.drop_in_place();
    }

    static INFO: TypeInfoWrapper = TypeInfoWrapper(GTypeInfo {
        class_size: size_of::<KimeIMContextClass>() as _,
        base_init: None,
        base_finalize: None,
        class_init: Some(im_context_class_init),
        class_finalize: Some(im_context_class_finalize),
        class_data: ptr::null(),
        instance_size: size_of::<KimeIMContext>() as _,
        n_preallocs: 0,
        instance_init: Some(im_context_instance_init),
        value_table: ptr::null(),
    });

    KIME_TYPE_IM_CONTEXT.get_or_init(|| {
        if module.is_null() {
            g_type_register_static(gtk_im_context_get_type(), cs!("KimeImContext"), &INFO.0, 0)
        } else {
            g_type_module_register_type(
                module,
                gtk_im_context_get_type(),
                cs!("KimeIMContext"),
                &INFO.0,
                0,
            )
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn im_module_init(module: *mut GTypeModule) {
    g_type_module_use(module);
    register_module(module);
}

#[no_mangle]
pub unsafe extern "C" fn im_module_exit() {}

#[no_mangle]
pub unsafe extern "C" fn im_module_list(
    contexts: *mut *const *const GtkIMContextInfo,
    n_contexts: *mut c_int,
) {
    static INFO: ContextInfoWrapper = ContextInfoWrapper(GtkIMContextInfo {
        context_id: cs!("kime"),
        context_name: cs!("Kime (Korean IME)"),
        domain: cs!("kime"),
        domain_dirname: cs!("/usr/share/locale"),
        default_locales: cs!("ko:*"),
    });

    static INFOS: &[&ContextInfoWrapper] = &[&INFO];

    // SAFETY: *const &ContextInfoWrapper -> *const *const GtkIMContextInfo
    // & == *const, ContextInfoWrapper == GtkIMContextInfo(transparent)
    contexts.write(INFOS.as_ptr().cast());
    n_contexts.write(INFOS.len() as _);
}

#[no_mangle]
pub unsafe extern "C" fn im_module_create(
    context_id: *const c_char,
) -> Option<ptr::NonNull<GtkIMContext>> {
    if !context_id.is_null() && g_strcmp0(context_id, cs!("kime")) == 0 {
        let ty = *KIME_TYPE_IM_CONTEXT.get()?;
        let obj = g_object_new(ty, ptr::null());
        ptr::NonNull::new(g_type_check_instance_cast(obj.cast(), ty).cast())
    } else {
        None
    }
}
