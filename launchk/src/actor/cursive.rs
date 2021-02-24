use cursive::Cursive;
use actix::{Actor, Context};

struct CursiveActor {
    siv: Cursive
}

impl Default for CursiveActor {
    fn default() -> Self {
        CursiveActor { siv: cursive::default() }
    }
}

impl Actor for CursiveActor {
    type Context = Context<Self>;
}
