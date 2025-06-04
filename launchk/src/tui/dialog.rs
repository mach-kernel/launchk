use std::sync::mpsc::Sender;

use cursive::Cursive;
use cursive::{
    view::Margins,
    views::{Dialog, TextView},
};

use crate::tui::omnibox::command::OmniboxCommand;
use crate::tui::omnibox::command::OMNIBOX_COMMANDS;
use crate::tui::omnibox::view::OmniboxEvent;
use crate::tui::root::CbSinkMessage;
use xpc_sys::csr::{csr_check, CsrConfig};

/// XPC "error" key can be present with no failure..."notice"?
pub fn show_notice(msg: String, title: Option<String>) -> CbSinkMessage {
    let cl = |siv: &mut Cursive| {
        let dialog = Dialog::around(TextView::new(msg))
            .button("Ok", |s| {
                s.pop_layer();
            })
            .title(title.unwrap_or("Notice".to_string()));

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
                commands
                    .iter()
                    .try_for_each(|c| tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must send commands");

                s.pop_layer();
            })
            .dismiss_button("No")
            .title("Notice");

        siv.add_layer(ask);
    };

    Box::new(cl)
}

pub fn show_csr_info() -> CbSinkMessage {
    let csr_flags = (0..11)
        .map(|s| {
            let mask = CsrConfig::from_bits(1 << s).expect("Must be in CsrConfig");
            format!("{:?}: {}", mask, unsafe { csr_check(mask.bits()) } == 0)
        })
        .collect::<Vec<String>>();

    Box::new(move |siv| {
        siv.add_layer(
            Dialog::new()
                .title("CSR Info")
                .content(TextView::new(csr_flags.join("\n")))
                .dismiss_button("OK")
                .padding(Margins::trbl(4, 4, 4, 4)),
        )
    })
}

pub fn show_help() -> CbSinkMessage {
    let commands = OMNIBOX_COMMANDS
        .iter()
        .map(|(cmd, desc, _)| {
            format!(
                "{:<15}: {}",
                cmd,
                desc.chars().filter(|c| c.is_ascii()).collect::<String>()
            )
        })
        .collect::<Vec<String>>();

    Box::new(move |siv| {
        siv.add_layer(
            Dialog::new()
                .title("Help")
                .content(TextView::new(commands.join("\n")))
                .dismiss_button("OK")
                .padding(Margins::trbl(4, 4, 4, 4)),
        )
    })
}
