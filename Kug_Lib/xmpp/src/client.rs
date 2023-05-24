use crate::user::User;
use crate::xmpp_stream::XMPPStream;
use minidom::Node;
use sasl::client::mechanisms::Plain;
use sasl::client::Mechanism;
use sasl::common::{ChannelBinding, Credentials};
use xmpp_parsers::message::Message;
use crate::packet::{Packet, PacketImporter};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::TcpStream;
use std::str::{self, FromStr};
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;
use xmpp_parsers::Element;

const PORT: i32 = 5222;
const RESOURCE_BIND: &str = "Kug";

#[derive(Debug)]
pub enum XMPPEvent {
    Message(Message),
    Test(Packet),
}

pub struct Client {
    user: User,
    stream: Option<XMPPStream>,
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(_) = self.stream {
            self.disconnect();
        }
    }
}

impl Client {
    pub fn new(user: User) -> Self {
        Self { user, stream: None }
    }

    fn get_host_ip(&self) -> String {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        let response = resolver
            .lookup_ip(&self.user.username.clone().domain())
            .unwrap();

        let address = response.iter().next().expect("no addresses returned!");

        match address {
            IpAddr::V4(addr) => addr.to_string(),
            IpAddr::V6(addr) => addr.to_string(),
        }
    }

    fn generate_resource_bind(&self) -> Element {
        let resource = Element::builder("resource", "")
            .append(RESOURCE_BIND)
            .build();

        let bind = Element::builder("bind", "urn:ietf:params:xml:ns:xmpp-bind")
            .append(resource)
            .build();

        let root = Element::builder("iq", "jabber:client")
            .attr("type", "set")
            .attr("id", "123") // TODO: ID's are required for 'iq' stanzas. Generate a random, unique id.
            .append(bind)
            .build();

        root

        // let bind_iq = "<iq xmlns='jabber:client' type='set' id='46725cf8-cbf8-4acf-aeba-dd2c678cc932'><bind xmlns=''><resource>Kug</resource></bind></iq>";
        // println!("[CLI] >>> {}", bind_iq);
    }

    pub fn connect(&mut self) {
        let host_addr = self.get_host_ip();
        println!("host ip is {}", host_addr);
        let tcp_stream = TcpStream::connect(format!("{}:{}", host_addr, PORT)).unwrap();
        let stream = XMPPStream::new(tcp_stream, self.user.clone());

        stream.start_stream();

        // TODO: Read features and apply any that is required or should be enabled (example: starttls)
        // Test server doesn't require any features, so let's just move on. Just need something working locally.
        stream.read();

        let credentials = Credentials::default()
            .with_username(self.user.username.clone().node().unwrap())
            .with_password(self.user.password.clone())
            .with_channel_binding(ChannelBinding::None);

        let mut sasl_cred = Plain::from_credentials(credentials).unwrap();
        let initial = sasl_cred.initial();
        let mut user_payload: Vec<u8> = vec![0; initial.len() * 4 / 3 + 4];

        let written = general_purpose::STANDARD
            .encode_slice(initial.as_slice(), &mut user_payload)
            .unwrap();
        user_payload.truncate(written);

        // TODO: Read mechanisms and connect via the best mechanism
        // We are just using PLAIN to see if our code works.

        let auth_packet = Packet::Stanza(
            Element::builder("auth", "urn:ietf:params:xml:ns:xmpp-sasl")
                .attr("mechanism", "PLAIN")
                .append(Node::Text(String::from(
                    str::from_utf8(user_payload.as_slice()).unwrap(),
                )))
                .build(),
        );

        stream.send(auth_packet);

        // TODO: When we use other authentication methods, we may run into a challenge (or multiple!)
        // Loop until we read a success stanza.
        // PLAIN shouldn't challenge though (when I tested)

        stream.read();

        // Start a new stream after we authenticate (RFC shows this in an example)
        // It should send a stream header, and features as a child in one stanza.

        stream.start_stream();

        let bind_iq = Packet::Stanza(self.generate_resource_bind());
        stream.send(bind_iq);

        stream.read();

        let presence_packet = Packet::Stanza(
            Element::builder("presence", "jabber:client")
                .append(Element::bare("show", ""))
                .build(),
        );

        stream.send(presence_packet);
        stream.read();

        self.stream = Some(stream);
    }

    pub fn disconnect(&mut self) {
        if let None = &self.stream {
            panic!("There is no stream. Did you forget to connect?");
        }

        let stream = self.stream.as_ref().unwrap();
        stream.send(Packet::End);

        stream.read();

        self.stream = None;
    }

    pub fn events(&mut self) -> Option<Vec<XMPPEvent>> {
        if let None = &self.stream {
            panic!("There is no stream. Did you forget to connect?");
        }

        let stream = self.stream.as_ref().unwrap();

        let packet = PacketImporter::import(stream.read());

        // Test server sends a iq, query with the namespace jabber:iq:version. So we should see the event fire once.
        Some(vec![XMPPEvent::Test(packet.unwrap())])
    }
}
