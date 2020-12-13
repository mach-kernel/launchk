use cursive::views::{Dialog, TextView};

mod launchd;

fn main() {
    let mut siv = cursive::default();

    // Creates a dialog with a single "Quit" button
    siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
                         .title("Cursive")
                         .button("Quit", |s| s.quit()));

    // Starts the event loop.
    siv.run();

    // println!("agents {}", launchd::list_agents());
}
