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
    use crate::lpnlib::{ElpsMsg::*, *};
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
                            vel: 72,
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
