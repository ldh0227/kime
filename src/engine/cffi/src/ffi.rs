/* automatically generated by rust-bindgen 0.56.0 */

pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __uint32_t = ::std::os::raw::c_uint;
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum InputResultType {
    Bypass = 0,
    ToggleHangul = 1,
    ClearPreedit = 2,
    Preedit = 3,
    Commit = 4,
    CommitBypass = 5,
    CommitPreedit = 6,
    CommitCommit = 7,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Config {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InputEngine {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InputResult {
    pub ty: InputResultType,
    pub char1: u32,
    pub char2: u32,
}
#[test]
fn bindgen_test_layout_InputResult() {
    assert_eq!(
        ::std::mem::size_of::<InputResult>(),
        12usize,
        concat!("Size of: ", stringify!(InputResult))
    );
    assert_eq!(
        ::std::mem::align_of::<InputResult>(),
        4usize,
        concat!("Alignment of ", stringify!(InputResult))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<InputResult>())).ty as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(InputResult),
            "::",
            stringify!(ty)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<InputResult>())).char1 as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(InputResult),
            "::",
            stringify!(char1)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<InputResult>())).char2 as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(InputResult),
            "::",
            stringify!(char2)
        )
    );
}
pub type ModifierState = u32;
extern "C" {
    #[doc = " Create new engine"]
    pub fn kime_engine_new() -> *mut InputEngine;
}
extern "C" {
    #[doc = " Delete engine"]
    pub fn kime_engine_delete(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Is hangul enabled"]
    pub fn kime_engine_is_hangul_enabled(engine: *const InputEngine) -> u32;
}
extern "C" {
    pub fn kime_engine_focus_in(engine: *mut InputEngine);
}
extern "C" {
    pub fn kime_engine_focus_out(engine: *mut InputEngine);
}
extern "C" {
    pub fn kime_engine_update_preedit(engine: *mut InputEngine, x: u32, y: u32, ch: u32);
}
extern "C" {
    pub fn kime_engine_remove_preedit(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Get preedit_char of engine"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " valid ucs4 char NULL to represent empty"]
    pub fn kime_engine_preedit_char(engine: *const InputEngine) -> u32;
}
extern "C" {
    #[doc = " Reset preedit state then returm commit char"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " valid ucs4 char NULL to represent empty"]
    pub fn kime_engine_reset(engine: *mut InputEngine) -> u32;
}
extern "C" {
    #[doc = " Press key when modifier state"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " input result"]
    pub fn kime_engine_press_key(
        engine: *mut InputEngine,
        config: *const Config,
        hardware_code: u16,
        state: ModifierState,
    ) -> InputResult;
}
extern "C" {
    #[doc = " Load config from local file"]
    pub fn kime_config_load() -> *mut Config;
}
extern "C" {
    #[doc = " Delete config"]
    pub fn kime_config_delete(config: *mut Config);
}
extern "C" {
    #[doc = " Get xim_preedit_font config"]
    #[doc = " name only valid while config is live"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " utf-8 string when len"]
    pub fn kime_config_xim_preedit_font(
        config: *const Config,
        name: *mut *const u8,
        len: *mut usize,
        font_size: *mut f64,
    );
}
