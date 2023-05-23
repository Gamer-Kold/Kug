extern crate xmpp;

use xmpp::client::Client;
use xmpp::user::User;

fn main() {
    let user = User::new("admin@127.0.0.1", "admin");
    let mut client = Client::new(user);

    client.connect();

    println!("Hello, world!");

    client.disconnect();
}
