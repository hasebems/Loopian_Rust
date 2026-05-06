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
                            note: 60,
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
