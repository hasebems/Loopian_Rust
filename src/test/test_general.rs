use std::sync::mpsc::TryRecvError;
use crate::lpnlib::{*, ElpsMsg::*};

#[test]
fn general1() {
    let (txmsg, _rxmsg) = std::sync::mpsc::channel();
    let (_txui, rxui) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg, rxui);

    assert_eq!(cmd.set_and_responce("ABC"), Some("what?".to_string()));
}
#[test]
fn pedal() {
    let (txmsg, rxmsg) = std::sync::mpsc::channel();
    let (_txui, rxui) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg, rxui);

    assert_eq!(cmd.set_and_responce("[d].dmp(off)"), Some("Set Phrase!".to_string()));
    loop {
        // message 受信処理
        match rxmsg.try_recv() {
            Ok(n)  => {
                match n {
                    Phr(_m0, _m1, evt) => {
                        assert_eq!(evt[0],PhrEvt{mtype:TYPE_NOTE, tick: 0, dur: 440, note: 60, vel: 72, trns: 0 });
                    }
                    Ana(_m, evt) => {
                        assert_eq!(evt[0],AnaEvt{mtype:TYPE_BEAT, tick: 0, dur: 480, note: 60, cnt: 1, atype: 0});
                        assert_eq!(evt[1],AnaEvt{mtype:TYPE_EXP, tick:0, dur:0, note:0, cnt:0, atype:NOPED});
                    }
                    _ => {},
                }
            },
            Err(TryRecvError::Disconnected) => assert!(false),
            Err(TryRecvError::Empty) => break,
        }
    }
}