use std::collections::BTreeMap;

use enumset::EnumSet;
use kime_engine_core::{
    Addon, Config, Hotkey, InputEngine, InputResult, Key, KeyCode::*, RawConfig,
};

fn default_config() -> Config {
    Config::from_raw_config(
        RawConfig {
            layout: "dubeolsik".into(),
            ..Default::default()
        },
        None,
    )
}

fn addon_config(addon: EnumSet<Addon>) -> Config {
    Config::from_raw_config(
        RawConfig {
            layout: "dubeolsik".into(),
            layout_addons: std::iter::once(("dubeolsik".into(), addon)).collect(),
            ..Default::default()
        },
        None,
    )
}

fn hotkey_config(hotkeys: BTreeMap<Key, Hotkey>) -> Config {
    Config::from_raw_config(
        RawConfig {
            layout: "dubeolsik".into(),
            hotkeys,
            ..Default::default()
        },
        None,
    )
}

#[track_caller]
fn test_input_impl(config: &Config, word_commit: bool, keys: &[(Key, &str, &str)]) {
    let mut engine = InputEngine::new(word_commit);

    engine.set_hangul_enable(true);

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

        eprintln!("Ret: {:?}", ret);

        if ret.contains(InputResult::HAS_PREEDIT) {
            assert_eq!(preedit, engine.preedit_str());
        } else {
            assert!(preedit.is_empty());
        }

        if !ret.contains(InputResult::CONSUMED) {
            assert_eq!(commit, format!("{}PASS", engine.commit_str()));
        } else if ret.intersects(InputResult::NEED_RESET | InputResult::NEED_FLUSH) {
            assert_eq!(commit, engine.commit_str());
        } else {
            assert!(commit.is_empty());
        }

        if ret.contains(InputResult::NEED_RESET) {
            engine.reset();
        } else if ret.contains(InputResult::NEED_FLUSH) {
            engine.flush();
        }
    }
}

#[track_caller]
fn test_input(keys: &[(Key, &str, &str)]) {
    test_input_impl(&default_config(), false, keys);
}

#[track_caller]
fn test_input_with_addon(keys: &[(Key, &str, &str)], addon: EnumSet<Addon>) {
    test_input_impl(&addon_config(addon), false, keys);
}

#[track_caller]
fn test_input_with_hotkey(keys: &[(Key, &str, &str)], hotkeys: BTreeMap<Key, Hotkey>) {
    test_input_impl(&hotkey_config(hotkeys), false, keys);
}

#[track_caller]
fn test_word_input(keys: &[(Key, &str, &str)]) {
    test_input_impl(&default_config(), true, keys);
}

#[test]
fn flexible_compose_order_addon() {
    test_input_with_addon(
        &[(Key::normal(K), "ㅏ", ""), (Key::normal(R), "가", "")],
        EnumSet::only(Addon::FlexibleComposeOrder),
    );
}

#[test]
fn space_commit() {
    test_input_with_hotkey(
        &[
            (Key::normal(R), "ㄱ", ""),
            (Key::normal(K), "가", ""),
            (Key::normal(Space), "", "가"),
            (Key::normal(S), "ㄴ", ""),
            (Key::normal(K), "나", ""),
            (Key::normal(Space), "", "나"),
            (Key::normal(Space), "", "PASS"),
        ],
        std::iter::once((
            Key::normal(Space),
            Hotkey::new(
                kime_engine_core::HotkeyBehavior::Commit,
                kime_engine_core::HotkeyResult::ConsumeIfProcessed,
            ),
        ))
        .collect(),
    )
}

#[test]
fn word_hello() {
    test_word_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(S), "안ㄴ", ""),
        (Key::normal(U), "안녀", ""),
        (Key::normal(D), "안녕", ""),
        (Key::normal(Esc), "", "안녕PASS"),
    ])
}

// issue #310
#[test]
fn hangul_change_preedit() {
    test_input(&[(Key::normal(R), "ㄱ", ""), (Key::normal(Hangul), "ㄱ", "")]);
}

#[test]
fn esc() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(Esc), "", "ㄱPASS"),
        (Key::normal(R), "", "PASS"),
    ]);
}

#[test]
fn strict_typing_order() {
    test_input(&[(Key::normal(K), "ㅏ", ""), (Key::normal(R), "ㄱ", "ㅏ")])
}

#[test]
fn next_jaum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(D), "앙", ""),
        (Key::normal(E), "ㄷ", "앙"),
    ])
}

#[test]
fn next_ssangjaum() {
    test_input(&[
        (Key::normal(A), "ㅁ", ""),
        (Key::normal(K), "마", ""),
        (Key::shift(T), "맜", ""),
        (Key::normal(K), "싸", "마"),
    ])
}

#[test]
fn not_com_moum_when_continue() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(D), "옹", ""),
        (Key::normal(K), "아", "오"),
    ]);
}

#[test]
fn com_moum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
    ]);
}

#[test]
fn number() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
        (Key::normal(One), "", "ㄷ1"),
    ]);
}

#[test]
fn exclamation_mark() {
    test_input(&[(Key::shift(R), "ㄲ", ""), (Key::shift(One), "", "ㄲ!")]);
}

#[test]
fn backspace() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(K), "가", ""),
        (Key::normal(D), "강", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Q), "갑", ""),
        (Key::normal(T), "값", ""),
        (Key::normal(Backspace), "갑", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Backspace), "ㄱ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(R), "ㄱ", ""),
    ])
}

#[test]
fn compose_jong() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(J), "어", ""),
        (Key::normal(Q), "업", ""),
        (Key::normal(T), "없", ""),
    ])
}

#[test]
fn backspace_moum_compose() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
    ])
}
