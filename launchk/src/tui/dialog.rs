use std::sync::mpsc::Sender;

use cursive::Cursive;
use cursive::traits::Scrollable;
use cursive::{
    theme::Effect,
    view::Margins,
    views::{Dialog, DummyView, LinearLayout, RadioGroup, TextView},
};

use crate::launchd::entry_status::{get_entry_status, LaunchdEntryStatus};
use crate::tui::omnibox::command::OMNIBOX_COMMANDS;
use crate::tui::omnibox::view::OmniboxEvent;
use crate::tui::root::CbSinkMessage;
use crate::{
    launchd::enums::{DomainType, SessionType},
    tui::omnibox::command::OmniboxCommand,
};
use xpc_sys::csr::{csr_check, CsrConfig};

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
                commands
                    .iter()
                    .try_for_each(|c| tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must sent commands");

                s.pop_layer();
            })
            .dismiss_button("No")
            .title("Notice");

        siv.add_layer(ask);
    };

    Box::new(cl)
}

pub fn domain_session_prompt<S: Into<String>>(
    label: S,
    domain_only: bool,
    tx: Sender<OmniboxEvent>,
    f: fn(DomainType, Option<SessionType>) -> Vec<OmniboxCommand>,
) -> CbSinkMessage {
    let LaunchdEntryStatus {
        limit_load_to_session_type,
        domain,
        ..
    } = get_entry_status(label);

    let cl = move |siv: &mut Cursive| {
        let mut domain_group: RadioGroup<DomainType> = RadioGroup::new();
        let mut st_group: RadioGroup<SessionType> = RadioGroup::new();

        // Build domain type list
        let mut domain_type_layout = LinearLayout::vertical()
            .child(TextView::new("Domain Type").effect(Effect::Bold))
            .child(DummyView);

        for d in DomainType::System as u64..DomainType::Unknown as u64 {
            let as_domain: DomainType = d.into();
            let mut button =
                domain_group.button(as_domain.clone(), format!("{}: {}", d, &as_domain));
            if as_domain == domain {
                button = button.selected();
            }

            domain_type_layout = domain_type_layout.child(button);
        }

        let mut session_type_layout = LinearLayout::vertical();

        if !domain_only {
            session_type_layout = session_type_layout
                .child(TextView::new("Session Type").effect(Effect::Bold))
                .child(DummyView);

            for s in SessionType::Aqua as u64..SessionType::Unknown as u64 {
                let as_session: SessionType = s.into();

                let mut button = st_group.button(as_session.clone(), as_session.to_string());
                if as_session == limit_load_to_session_type {
                    button = button.selected();
                }
                session_type_layout = session_type_layout.child(button);
            }
        }

        let layout = LinearLayout::horizontal()
            .child(domain_type_layout)
            .child(session_type_layout);

        let ask = Dialog::new()
            .title("Please select to continue")
            .content(layout)
            .button("OK", move |s| {
                let dt = domain_group.selection().as_ref().clone();
                let st = if domain_only {
                    None
                } else {
                    Some(st_group.selection().as_ref().clone())
                };

                f(dt, st)
                    .iter()
                    .try_for_each(|c| tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must send commands");

                s.pop_layer();
            })
            .dismiss_button("Cancel")
            .padding(Margins::trbl(5, 5, 5, 5));

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
                .padding(Margins::trbl(2, 2, 2, 2)),
        )
    })
}

pub fn show_help() -> CbSinkMessage {
    let commands = OMNIBOX_COMMANDS
        .iter()
        .map(|(cmd, desc, _)| format!("{}: {}", cmd, desc))
        .collect::<Vec<String>>();

    Box::new(move |siv| {
        siv.add_layer(
            Dialog::new()
                .title("Help")
                .content(TextView::new(commands.join("\n")))
                .dismiss_button("OK")
                .padding(Margins::trbl(2, 2, 2, 2)),
        )
    })
}
