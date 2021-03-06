use kime_engine_core::{Config, InputEngine, InputResult, Key, KeyCode::*, RawConfig};

#[track_caller]
fn test_input(keys: &[(Key, &str, &str)]) {
    let config = Config::from_raw_config(
        RawConfig {
            layout: "sebeolsik-sin1995".into(),
            ..Default::default()
        },
        None,
    );

    let mut engine = InputEngine::new(false);

    engine.set_hangul_enable(true);

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

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

#[test]
fn simple() {
    test_input(&[(Key::normal(U), "ㄷ", ""), (Key::normal(Q), "듸", "")]);
}

#[test]
fn mu() {
    test_input(&[(Key::normal(I), "ㅁ", ""), (Key::normal(I), "무", "")]);
}

#[test]
fn hello() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(H), "ㄴ", "안"),
        (Key::normal(E), "녀", ""),
        (Key::normal(A), "녕", ""),
        (Key::normal(M), "ㅎ", "녕"),
        (Key::normal(F), "하", ""),
        (Key::normal(N), "ㅅ", "하"),
        (Key::normal(C), "세", ""),
        (Key::normal(J), "ㅇ", "세"),
        (Key::normal(X), "요", ""),
    ]);
}

// issue #295
#[test]
fn compose_jungseong() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(O), "우", ""),
        (Key::normal(C), "웨", ""),
        (Key::normal(S), "웬", ""),
        (Key::normal(J), "ㅇ", "웬"),
        (Key::normal(P), "오", ""),
        (Key::normal(D), "외", ""),
    ]);
}

#[test]
fn dont_compose_jungseong() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(B), "우", ""),
        (Key::normal(C), "웇", ""),
        (Key::normal(J), "ㅇ", "웇"),
        (Key::normal(V), "오", ""),
        (Key::normal(D), "옿", ""),
    ]);
}

#[test]
fn chocolate() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(O), "우", ""),
        (Key::normal(C), "웨", ""),
        (Key::normal(S), "웬", ""),
        (Key::normal(O), "ㅊ", "웬"),
        (Key::normal(V), "초", ""),
        (Key::normal(Slash), "ㅋ", "초"),
        (Key::normal(V), "코", ""),
        (Key::normal(W), "콜", ""),
        (Key::normal(Y), "ㄹ", "콜"),
        (Key::normal(D), "리", ""),
        (Key::normal(Q), "릿", ""),
    ]);
}
