use std::sync::mpsc::Sender;

use cursive::Cursive;
use cursive::{
    theme::Effect,
    view::Margins,
    views::{Dialog, DummyView, LinearLayout, RadioGroup, TextView},
};

use crate::tui::omnibox::view::OmniboxEvent;
use crate::tui::root::CbSinkMessage;
use crate::{
    launchd::enums::{DomainType, SessionType},
    tui::omnibox::command::OmniboxCommand,
};
use crate::launchd::entry_status::{get_entry_status, LaunchdEntryStatus};

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
    let cl = move |siv: &mut Cursive| {
        let mut domain_group: RadioGroup<DomainType> = RadioGroup::new();
        let mut st_group: RadioGroup<SessionType> = RadioGroup::new();

        // Build domain type list
        let mut domain_type_layout = LinearLayout::vertical()
            .child(TextView::new("Domain Type").effect(Effect::Bold))
            .child(DummyView);

        let LaunchdEntryStatus {
            limit_load_to_session_type,
            domain,
            ..
        } = get_entry_status(&label);

        for d in DomainType::System..DomainType::RequestorDomain {
            let mut button = domain_group.button(d, format!("{}: {}", &d as u64, &d));
            if d == domain {
                button = button.selected();
            }

            domain_type_layout = domain_type_layout.child(button);
        }

        let mut session_type_layout = LinearLayout::vertical();

        if !domain_only {
            session_type_layout = session_type_layout
                .child(TextView::new("Session Type").effect(Effect::Bold))
                .child(DummyView);

            for s in SessionType::Aqua..SessionType::Unknown {
                let mut button = st_group.button(s, s.to_string());
                if s == limit_load_to_session_type {
                    button = button.selected();
                }
                session_type_layout = session_type_layout.child(button);
            }
        }

        let mut layout = LinearLayout::horizontal()
            .child(domain_type_layout)
            .child(session_type_layout);

        let ask = Dialog::new()
            .title("Please choose")
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
