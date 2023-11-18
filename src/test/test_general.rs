#[test]
fn general1() {
    let (txmsg, _rxmsg) = std::sync::mpsc::channel();
    let (_txui, rxui) = std::sync::mpsc::channel();
    let mut cmd = crate::cmd::cmdparse::LoopianCmd::new(txmsg, rxui);

    assert_eq!(cmd.set_and_responce("ABC"), Some("what?".to_string()));
}