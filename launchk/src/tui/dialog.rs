use std::sync::mpsc::Sender;

use cursive::{CbSink, views::{Dialog, DummyView, LinearLayout, RadioGroup, TextView}};
use cursive::Cursive;

use crate::{launchd::query::{DomainType, LimitLoadToSessionType}, tui::omnibox::command::OmniboxCommand};
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
                commands
                    .iter()
                    .try_for_each(|c| tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must sent commands");

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

pub fn domain_session_prompt(
    tx: Sender<OmniboxEvent>,
    f: fn(DomainType, LimitLoadToSessionType) -> Vec<OmniboxCommand>,
) -> CbSinkMessage {
    let cl = move |siv: &mut Cursive| {
        let mut domain_group: RadioGroup<DomainType> = RadioGroup::new();
        let mut lltst_group: RadioGroup<LimitLoadToSessionType> = RadioGroup::new();

        let ask = Dialog::new()
            .title("Choose domain and LimitLoadToSessionType")
            .content(
            LinearLayout::horizontal()
                    .child(
                    LinearLayout::vertical()
                            .child(TextView::new("Domain Type"))
                            .child(domain_group.button(DomainType::System, "System (1)"))
                            .child(domain_group.button(DomainType::User, "User (2)"))
                            .child(domain_group.button(DomainType::UserLogin, "UserLogin (3)"))
                            .child(domain_group.button(DomainType::Session, "Session (4)"))
                            // TODO: Ask for handle
                            .child(domain_group.button(DomainType::PID, "PID (5)").disabled())
                            .child(domain_group.button(DomainType::RequestorUserDomain, "Requestor User Domain (6)"))
                            // TODO: Is this a sane default?
                            .child(domain_group.button(DomainType::RequestorDomain, "Requestor Domain (7)").selected())
                    )
                    .child(DummyView)
                    .child(
                    LinearLayout::vertical()
                            .child(TextView::new("Domain Type"))
                            .child(lltst_group.button(LimitLoadToSessionType::Aqua, LimitLoadToSessionType::Aqua.to_string()))
                            .child(lltst_group.button(LimitLoadToSessionType::StandardIO, LimitLoadToSessionType::StandardIO.to_string()))
                            .child(lltst_group.button(LimitLoadToSessionType::Background, LimitLoadToSessionType::Background.to_string()))
                            .child(lltst_group.button(LimitLoadToSessionType::LoginWindow, LimitLoadToSessionType::LoginWindow.to_string()))
                            .child(lltst_group.button(LimitLoadToSessionType::System, LimitLoadToSessionType::System.to_string()))
                    ),
            )
            .button("OK", move |s| {
                let dt = domain_group.selection().as_ref().clone();
                let lltst = lltst_group.selection().as_ref().clone();

                f(dt, lltst)
                    .iter()
                    .try_for_each(|c| tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must sent commands");

                s.pop_layer();
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(ask);
    };

    Box::new(cl)
}