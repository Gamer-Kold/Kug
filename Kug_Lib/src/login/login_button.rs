use gdnative::{
    api::{Button, File, LineEdit},
    prelude::*,
};

use xmpp::user::User;
use serde_json::{self};

// use crate::classes::user::User;
#[derive(NativeClass)]
#[inherit(Button)]
pub struct LoginButton {
    username_field: Option<Ref<LineEdit>>,
    password_field: Option<Ref<LineEdit>>,
}

impl LoginButton {
    fn new(_base: &Button) -> Self {
        LoginButton {
            username_field: None,
            password_field: None,
        }
    }
}

#[methods]
impl LoginButton {
    #[method]
    fn _ready(&mut self, #[base] base: &Button) {
        let username_field = base
            .get_node("../UsernameField")
            .expect("There is no node called UsernameField in the button's parent node.");
        let username_field = unsafe { username_field.assume_safe() };
        let username_field = username_field
            .cast::<LineEdit>()
            .expect("UsernameField must be of type 'LineEdit'");

        self.username_field = Some(username_field.claim());

        let password_field = base
            .get_node("../PasswordField")
            .expect("There is no node called PasswordField in the button's parent node.");
        let password_field = unsafe { password_field.assume_safe() };
        let password_field = password_field
            .cast::<LineEdit>()
            .expect("PasswordField must be of type 'LineEdit'");

        self.password_field = Some(password_field.claim());
    }

    #[method]
    fn _pressed(&self, #[base] base: &Button) {
        let username = unsafe { self.username_field.unwrap().assume_safe().text() };
        let password = unsafe { self.password_field.unwrap().assume_safe().text() };

        godot_print!("Button pressed from Rust!");
        godot_print!("Username text: {}", username);
        godot_print!("Password text: {}", password);

        let user_object = User::new(&username.to_string(), &password.to_string());
        let json_str = serde_json::to_string(&user_object).unwrap();

        let user_file = File::new();
        user_file
            .open("user://user.json", File::WRITE)
            .expect("user://user.json must exist");
        user_file.store_string(json_str);
        user_file.flush();
        user_file.close();

        unsafe {
            base.get_tree().unwrap().assume_safe().change_scene("res://Main.tscn").unwrap();
        }
    }
}
