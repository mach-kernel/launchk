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

pub fn domain_session_prompt(
    domain_only: bool,
    tx: Sender<OmniboxEvent>,
    f: fn(DomainType, Option<SessionType>) -> Vec<OmniboxCommand>,
) -> CbSinkMessage {
    let cl = move |siv: &mut Cursive| {
        let mut domain_group: RadioGroup<DomainType> = RadioGroup::new();
        let mut st_group: RadioGroup<SessionType> = RadioGroup::new();

        let mut layout = LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(TextView::new("Domain Type").effect(Effect::Bold))
                    .child(DummyView)
                    .child(domain_group.button(DomainType::System, "1: System"))
                    .child(domain_group.button(DomainType::User, "2: User"))
                    .child(domain_group.button(DomainType::UserLogin, "3: UserLogin"))
                    .child(domain_group.button(DomainType::Session, "4: Session"))
                    // TODO: Ask for handle
                    .child(domain_group.button(DomainType::PID, "5: PID").disabled())
                    .child(domain_group.button(
                        DomainType::RequestorUserDomain,
                        "6: Requester User Domain",
                    ))
                    // TODO: Is this a sane default?
                    .child(
                        domain_group
                            .button(DomainType::RequestorDomain, "7: Requester Domain")
                            .selected(),
                    ),
            );

        if !domain_only {
            layout = layout.child(DummyView)
                .child(
                    LinearLayout::vertical()
                        .child(TextView::new("Session Type").effect(Effect::Bold))
                        .child(DummyView)
                        .child(
                            st_group.button(SessionType::Aqua, SessionType::Aqua.to_string()),
                        )
                        .child(st_group.button(
                            SessionType::StandardIO,
                            SessionType::StandardIO.to_string(),
                        ))
                        .child(st_group.button(
                            SessionType::Background,
                            SessionType::Background.to_string(),
                        ))
                        .child(st_group.button(
                            SessionType::LoginWindow,
                            SessionType::LoginWindow.to_string(),
                        ))
                        .child(
                            st_group
                                .button(SessionType::System, SessionType::System.to_string()),
                        ),
                );
        }

        let mut ask = Dialog::new()
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
