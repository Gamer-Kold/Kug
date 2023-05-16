#![allow(unused_imports, dead_code)]

use gdnative::{prelude::*, api::File};
use std::{thread, sync::mpsc::{Receiver, Sender, self}};
use serde_json::{self};

use crate::classes::user::User;

#[derive(Debug)]
enum CommunicationPacket {
    // Main thread (Godot) > Client thread (XMPP)
    SendMessage,
    SendUserInfo(User),

    // Client thread (XMPP) > Main thread (Godot)
    GetUserInfo,
    ContactFound,
    MessageReceived,
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Client{
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<CommunicationPacket>>,
    receiver: Option<Receiver<CommunicationPacket>>,

    user: User
}

impl Client {
    fn new(_base: &Node) -> Self {
        Client {
            thread: None,
            sender: None,
            receiver: None,
            user: User::default(),
        }
    }
    fn start_thread(&mut self, channels: (Sender<CommunicationPacket>, Receiver<CommunicationPacket>)) {
        self.thread = Some(thread::spawn(move || {
            if let Err(err) = channels.0.send(CommunicationPacket::GetUserInfo) {
                godot_print!("Thread sender channel threw error: {}", err);
            };

            let user: User;

            loop {
                godot_print!("Trying to get user packet.");
                if let Ok(packet) = channels.1.recv() {
                    if let CommunicationPacket::SendUserInfo(p_user) = packet {
                        godot_print!("Got user packet!");
                        user = p_user;
                        break;
                    }
                };
            }

            godot_print!("Jid is {}", user.username);
        }));
    }
}

#[methods]
impl Client {
    #[method]
    fn _ready(&mut self, #[base] _base: &Node) {
        // Load the user

        let user_reader = File::new();
        user_reader.open("user://user.json", File::READ)
            .expect("user://user.json must exist");
        let content = user_reader.get_as_text(true).to_string();
        self.user = serde_json::from_str(&content).expect("Could not read user.json");

        let channel_to_thread = mpsc::channel::<CommunicationPacket>();
        let channel_from_thread = mpsc::channel::<CommunicationPacket>();
        self.start_thread((channel_from_thread.0, channel_to_thread.1));

        self.sender = Some(channel_to_thread.0);
        self.receiver = Some(channel_from_thread.1);
    }

    #[method]
    fn _process(&self, #[base] _base: &Node, _delta: f32) {
        if let Ok(packet) = self.receiver.as_ref().unwrap().recv() {
            match packet {
                CommunicationPacket::GetUserInfo => {
                    self.sender.as_ref().unwrap().send(CommunicationPacket::SendUserInfo(self.user.clone())).unwrap();
                },
                _ => godot_print!("Got unknown packet: {:?}", packet),
            }
        }
    }
}