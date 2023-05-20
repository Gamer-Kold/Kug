extern crate xmpp;

use xmpp::client::Client;
use xmpp::user::User;

fn main() {
    let user = User::new("admin@localhost", "admin");
    let client = Client::new(user);

    client.connect();
    // println!("Hello, world!");
}