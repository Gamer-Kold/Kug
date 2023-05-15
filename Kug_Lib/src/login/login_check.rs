use gdnative::{
    api::File,
    prelude::*
};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct LoginCheck;

impl LoginCheck {
    fn new(_base: &Node) -> Self {
        LoginCheck
    }
}

#[methods]
impl LoginCheck {
    #[method]
    fn _ready(&self, #[base] base: &Node) {
        godot_print!("Checking if user.json exists...");

        let file_check = File::new();
        if file_check.file_exists("user://user.json") {
            godot_print!("User exists!");
            unsafe {
                base.get_tree().unwrap().assume_safe().change_scene("res://Main.tscn").unwrap();
            }
        } else {
            godot_print!("User does not exist!");
        }
    }
}