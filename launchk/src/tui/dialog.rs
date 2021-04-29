use std::sync::mpsc::Sender;

use cursive::views::{Dialog, TextView};
use cursive::Cursive;

use crate::tui::omnibox::command::OmniboxCommand;
use crate::tui::omnibox::view::OmniboxEvent;
use crate::tui::root::CbSinkMessage;

/// The XPC error key sometimes contains information that is not necessarily a failure,
/// so let's just call it "Notice" until we figure out what to do next?
pub fn show_error(err: String) -> CbSinkMessage {
    let cl = |siv: &mut Cursive| {
        let dialog = Dialog::around(TextView::new(err))
            .button("Ok", |s| {
                s.pop_layer();
            })
            .title("Notice");

        siv.add_layer(dialog);
    };

    Box::new(cl)
}

/// OmniboxCommand::Prompt(msg, followup commands)
pub fn show_prompt(
    tx: Sender<OmniboxEvent>,
    prompt: String,
    commands: Vec<OmniboxCommand>,
) -> CbSinkMessage {
    let cl = move |siv: &mut Cursive| {
        let ask = Dialog::around(TextView::new(prompt.clone()))
            .button("Yes", move |s| {
                for cmd in &commands {
                    tx.send(OmniboxEvent::Command(cmd.clone()))
                        .expect("Must send command");
                }
                s.pop_layer();
            })
            .button("No", |s| {
                s.pop_layer();
            })
            .title("Notice");

        siv.add_layer(ask);
    };

    Box::new(cl)
}
