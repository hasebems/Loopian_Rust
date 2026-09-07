#[test]
fn general1() {
    let (txmsg, _rxmsg) = std::sync::mpsc::channel();
    //let (_txui, rxui) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("ABC").unwrap().0,
        "what?".to_string()
    );
}
#[test]
fn pedal() {
    use crate::common::lpnlib::{ElpsMsg::*, *};
    use std::sync::mpsc::TryRecvError;

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    //let (_txui, rxui) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("[d].dmp(off)").unwrap().0,
        "Set Phrase!".to_string()
    );
    loop {
        // message 受信処理
        match rxmsg.try_recv() {
            Ok(n) => match n {
                Ctrl(_m0) => {
                    break;
                }
                Phr(_m0, dt) => {
                    assert_eq!(
                        dt.evts[0],
                        PhrEvt::Note(NoteEvt {
                            tick: 0,
                            dur: 440,
                            note: NoteNum::Num(60),
                            floating: false,
                            amp: Amp::default(),
                            trns: TrnsType::Com,
                            artic: 100,
                        })
                    );
                }
                _ => {}
            },
            Err(TryRecvError::Disconnected) => panic!(),
            Err(TryRecvError::Empty) => break,
        }
    }
}

#[test]
fn shortcut_phrase_chain() {
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L1.[d].dmp(off)").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg
            && part == LEFT1 as i16
        {
            found = true;
            assert_eq!(dt.evts.len(), 1);
        }
    }
    assert!(found);
}

#[test]
fn additional_phrase_with_part_prefix() {
    // L1.[aaa]+ の後に、続きは素の [...] のみで閉じる。
    // 閉じる側の入力に part 指定を伴わなくても、開始時の L1 へ結合される。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L1.[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[m,f]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg
            && part == LEFT1 as i16
        {
            found = true;
            assert_eq!(dt.evts.len(), 4);
        }
    }
    assert!(found);
}

#[test]
fn additional_phrase_multi_part_no_double_concat() {
    // L.[...]+ のように複数パート一括指定でも、文字列が二重連結されない。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L.[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[m,f]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut left1_found = false;
    let mut left2_found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg {
            if part == LEFT1 as i16 {
                left1_found = true;
                assert_eq!(dt.evts.len(), 4);
            } else if part == LEFT2 as i16 {
                left2_found = true;
                assert_eq!(dt.evts.len(), 4);
            }
        }
    }
    assert!(left1_found && left2_found);
}

#[test]
fn additional_phrase_pending_mismatch_keeps_pending() {
    // 保留中に、明示的な宛先を伴う「閉じ」入力が来たらエラーになり、
    // 保留バッファ自体は破棄されない(素の [...] で正しく閉じ直せる)。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L1.[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    // 明示的な宛先(L2.)を伴う「閉じ」入力はエラー
    assert_eq!(
        cmd.put_and_get_responce("L2.[m,f]").unwrap().0,
        "Unfinished phrase input! Close it with a plain [...] first.".to_string()
    );
    // 保留は破棄されていないので、素の [...] で正しく閉じられる
    assert_eq!(
        cmd.put_and_get_responce("[m,f]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg
            && part == LEFT1 as i16
        {
            found = true;
            assert_eq!(dt.evts.len(), 4);
        }
    }
    assert!(found);
}

#[test]
fn additional_phrase_explicit_open_replaces_pending() {
    // 保留中に、別の明示的な宛先を伴う新規オープンが来たら、
    // 前の未完成バッファは破棄され、新しい方に差し替わる。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L1.[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("L2.[m,f,]+").unwrap().0,
        "Discarded unfinished phrase input, started a new one!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[s]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut left1_found = false;
    let mut left2_found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg {
            if part == LEFT1 as i16 {
                left1_found = true;
            } else if part == LEFT2 as i16 {
                left2_found = true;
                assert_eq!(dt.evts.len(), 3);
            }
        }
    }
    // L1 宛の未完成バッファは破棄されたので、送信されない
    assert!(!left1_found);
    assert!(left2_found);
}

#[test]
fn additional_phrase_chain_three_times() {
    // [aaa]+ → [bbb]+ → [ccc] のように、素の [...]+ を連続して
    // 何度でも同じバッファへ連結できる(多段連結)。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("L1.[d,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[m]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg
            && part == LEFT1 as i16
        {
            found = true;
            assert_eq!(dt.evts.len(), 3);
        }
    }
    assert!(found);
}

#[test]
fn measure_phrase_with_plus() {
    // @msr(M)=[...]+ も、他の経路と同じ汎用機構でそのまま動く。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("@msr(2)=[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[m,f]").unwrap().0,
        "Set Phrase!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Phr(part, dt) = msg
            && part == RIGHT1 as i16
        {
            found = true;
            assert_eq!(dt.vari, PhraseAs::Measure(2));
            assert_eq!(dt.evts.len(), 4);
        }
    }
    assert!(found);
}

#[test]
fn shared_variation_across_parts() {
    // @1=[...]+ → [...] で共有 Variation を設定した時点では、
    // どのパートの elapse にも送信されない。
    // Composition で {X@1} と参照された時点で、そのパートへ初めて送信される。
    use crate::common::lpnlib::{ElpsMsg::*, *};

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("@1=[d,r,]+").unwrap().0,
        "Keep Phrase as being unified phrase!".to_string()
    );
    assert_eq!(
        cmd.put_and_get_responce("[m,f]").unwrap().0,
        "Set Variation Phrase!".to_string()
    );
    // まだ elapse には送られていない
    assert!(rxmsg.try_recv().is_err());

    // デフォルトのカレントパート(right1)で Composition から @1 を参照する
    assert_eq!(
        cmd.put_and_get_responce("{X@1}").unwrap().0,
        "Set Composition!".to_string()
    );

    let mut found_cmp = false;
    let mut found_vari = false;
    while let Ok(msg) = rxmsg.try_recv() {
        match msg {
            Cmp(part, _dt) if part == RIGHT1 as i16 => found_cmp = true,
            Phr(part, dt) if part == RIGHT1 as i16 => {
                found_vari = true;
                assert_eq!(dt.vari, PhraseAs::Variation(1));
                assert_eq!(dt.evts.len(), 4);
            }
            _ => {}
        }
    }
    assert!(found_cmp);
    assert!(found_vari);
}

#[test]
fn flow_composition_shortcut() {
    use crate::common::lpnlib::ElpsMsg::*;

    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg);

    assert_eq!(
        cmd.put_and_get_responce("FLOW.{I,IV,V,I}").unwrap().0,
        "Set Composition!".to_string()
    );

    let mut found = false;
    while let Ok(msg) = rxmsg.try_recv() {
        if let Cmp(_part, dt) = msg {
            found = true;
            assert!(!dt.evts.is_empty());
        }
    }
    assert!(found);
}
