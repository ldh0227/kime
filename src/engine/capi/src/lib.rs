#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};

/// Create new engine
#[no_mangle]
pub extern "C" fn kime_engine_new() -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new()))
}

/// Delete engine
#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: *mut InputEngine) {
    drop(Box::from_raw(engine));
}

/// Is hangul enabled
#[no_mangle]
pub unsafe extern "C" fn kime_engine_is_hangul_enabled(engine: *const InputEngine) -> u32 {
    let engine = engine.as_ref().unwrap();

    engine.is_hangul_enabled().into()
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_focus_in(engine: *mut InputEngine) {
    let engine = engine.as_mut().unwrap();
    engine.focus_in();
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_focus_out(engine: *mut InputEngine) {
    let engine = engine.as_mut().unwrap();
    engine.focus_out();
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_update_preedit(
    engine: *mut InputEngine,
    x: u32,
    y: u32,
    ch: u32,
) {
    let engine = engine.as_mut().unwrap();
    engine.update_preedit(x, y, std::char::from_u32(ch).unwrap());
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_remove_preedit(engine: *mut InputEngine) {
    let engine = engine.as_mut().unwrap();
    engine.remove_preedit();
}

/// Get preedit_char of engine
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
#[no_mangle]
pub unsafe extern "C" fn kime_engine_preedit_char(engine: *const InputEngine) -> u32 {
    let engine = engine.as_ref().unwrap();

    engine.preedit_char() as u32
}

/// Reset preedit state then returm commit char
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
#[no_mangle]
pub unsafe extern "C" fn kime_engine_reset(engine: *mut InputEngine) -> u32 {
    let engine = engine.as_mut().unwrap();
    engine.reset() as u32
}

/// Press key when modifier state
///
/// ## Return
///
/// input result
#[no_mangle]
pub unsafe extern "C" fn kime_engine_press_key(
    engine: *mut InputEngine,
    config: *const Config,
    hardware_code: u16,
    state: ModifierState,
) -> InputResult {
    let engine = engine.as_mut().unwrap();
    let config = config.as_ref().unwrap();

    engine.press_key_code(hardware_code, state, config)
}

/// Load config from local file
#[no_mangle]
pub extern "C" fn kime_config_load() -> *mut Config {
    Box::into_raw(Box::new(Config::load_from_config_dir().unwrap_or_default()))
}

/// Delete config
#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *mut Config) {
    drop(Box::from_raw(config));
}
