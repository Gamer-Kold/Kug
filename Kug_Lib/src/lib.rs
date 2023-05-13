use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld;

impl HelloWorld {
    fn new(_base: &Node) -> Self {
        HelloWorld
    }
}

#[methods]
impl HelloWorld {
    #[method]
    fn _ready(&self, #[base] base: &Node) {
        godot_print!("Hello world from node {}!", base.to_string());
    }
}

fn init(handle : InitHandle){
    handle.add_class::<HelloWorld>();
}

godot_init!(init);
