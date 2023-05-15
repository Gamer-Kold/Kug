#![allow(unused_imports, dead_code)]

use gdnative::{prelude::*, api::File};
use std::{thread, sync::mpsc::{Receiver, Sender, self}};
use xmpp::{ClientBuilder, ClientFeature, ClientType, Event};
use xmpp_parsers::{message::MessageType, Jid};
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
            godot_print!("Hello, world!");

            if let Err(err) = channels.0.send(CommunicationPacket::GetUserInfo) {
                godot_print!("Thread sender channel threw error: {}", err);
            };

            // loop {
            //     channels.1.recv();
            // }
            
            // let mut client = ClientBuilder::new(jid, password)
            //     .set_client(ClientType::Bot, "xmpp-rs")
            //     .set_website("https://gitlab.com/xmpp-rs/xmpp-rs")
            //     .set_default_nick("bot")
            //     .enable_feature(ClientFeature::Avatars)
            //     .enable_feature(ClientFeature::ContactList)
            //     .enable_feature(ClientFeature::JoinRooms)
            //     .build()
            //     .unwrap();

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

        godot_print!("Client loaded");
    }

    #[method]
    fn _process(&self, #[base] _base: &Node, _delta: f32) {
        if let Ok(packet) = self.receiver.as_ref().unwrap().recv() {
            match packet {
                CommunicationPacket::GetUserInfo => {
                    
                }
            }
        }
    }
}