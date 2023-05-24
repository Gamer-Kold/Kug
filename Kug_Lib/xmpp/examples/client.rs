extern crate xmpp;

use xmpp::client::Client;
use xmpp::user::User;

fn main() {
    let user = User::new("example@127.0.0.1", "example");
    let mut client = Client::new(user);

    client.connect();

    loop {
        if let Some(events) = client.events() {
            for event in events {
                println!("I got a event! {:?}", event);
            }
        }
    }
}
