use gdnative::prelude::*;

// pub mod classes; this has been deleted?
mod login;
mod app;

fn init(handle: InitHandle) {
    handle.add_class::<login::login_button::LoginButton>();
    handle.add_class::<login::login_check::LoginCheck>();
    handle.add_class::<app::client::Client>();
}

godot_init!(init);
