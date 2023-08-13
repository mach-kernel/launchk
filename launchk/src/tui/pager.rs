use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use super::root::CbSinkMessage;
use cursive::Cursive;
use clearscreen;

lazy_static! {
    static ref PAGER: String = env::var("PAGER").unwrap_or("less".to_string());
}

/// Show $PAGER (or less), write buf, and clear Cursive after exiting
pub fn show_pager(cbsink: &Sender<CbSinkMessage>, buf: &[u8]) -> Result<(), String> {
    clearscreen::clear().expect("Must clear screen");

    let mut pager = Command::new(&*PAGER)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // Broken pipe unless scroll to end, do not throw an error
    pager
        .stdin
        .take()
        .expect("Must get pager stdin")
        .write_all(buf)
        .unwrap_or(());

    let res = pager.wait().map_err(|e| e.to_string())?;

    cbsink
        .send(Box::new(Cursive::clear))
        .expect("Must clear after");

    if res.success() {
        Ok(())
    } else {
        Err(format!("{} exited {:?}", &*PAGER, res))
    }
}
