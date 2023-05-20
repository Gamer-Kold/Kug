use crate::user::User;
use sasl::common::{Credentials, ChannelBinding};
use sasl::client::Mechanism;
use sasl::client::mechanisms::Plain;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::str;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;
use crate::packet::Packet;
use xmpp_parsers::sasl::{Auth, Challenge, Failure, Mechanism as XMPPMechanism, Response, Success};
use base64::{Engine as _, engine::general_purpose};

const PORT: i32 = 5222;
const STREAM_VERSION: &str = "1.0";
const XML_NAMESPACE: &str = "jabber:client";
const XML_NAMESPACE_STREAM: &str = "http://etherx.jabber.org/streams";

pub struct Client {
    user: User
}

impl Client {
    pub fn new(user: User) -> Self {
        Self {
            user
        }
    }

    fn get_host_ip(&self) -> String {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        let response = resolver.lookup_ip(&self.user.username.clone().domain()).unwrap();

        let address = response.iter().next().expect("no addresses returned!");

        match address {
            IpAddr::V4(addr) => addr.to_string(),
            IpAddr::V6(addr) => addr.to_string(),
        }
    }

    pub fn connect(&self) {
        let host_addr = self.get_host_ip();
        println!("host ip is {}", host_addr);
        let stream = TcpStream::connect(format!("{}:{}", host_addr, PORT)).unwrap();
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);

        let stream_start_packet = Packet::Start(HashMap::from([
            (String::from("from"), self.user.username.to_string()),
            (String::from("to"), self.user.username.clone().domain()),
            (String::from("version"), String::from(STREAM_VERSION)),
            (String::from("xmlns"), String::from(XML_NAMESPACE)),
            (String::from("xmlns:stream"), String::from(XML_NAMESPACE_STREAM)),
        ]));

        let start_stream = crate::packet::PacketExporter::export(stream_start_packet).unwrap();

        println!("[CLI] >>> {start_stream}");

        writer.write(start_stream.as_bytes()).unwrap();
        writer.flush().unwrap();

        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

        // TODO: Add support for STARSSL
        // Test server doesn't require any features, so let's just move on. Just need something working locally.
        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

        let credentials = Credentials::default()
            .with_username(self.user.username.clone().node().unwrap())
            .with_password(self.user.password.clone())
            .with_channel_binding(ChannelBinding::None);

        let mut sasl_cred = Plain::from_credentials(credentials).unwrap();
        let initial = sasl_cred.initial();
        let mut user_payload: Vec<u8> = vec![0; initial.len() * 4 / 3 + 4];

        let written = general_purpose::STANDARD.encode_slice(initial.as_slice(), &mut user_payload).unwrap();
        user_payload.truncate(written);

        writer.write(b"<auth xmlns='urn:ietf:params:xml:ns:xmpp-sasl' mechanism='PLAIN'>").unwrap();
        writer.write(&user_payload).unwrap();
        writer.write(b"</auth>").unwrap();
        println!("[CLI] >>> {}", str::from_utf8(writer.buffer()).unwrap());
        writer.flush().unwrap();

        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

        writer.write(start_stream.as_bytes()).unwrap();
        writer.flush().unwrap();
        println!("[CLI] >>> {start_stream}");

        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

        writer.write(b"<presence><show/></presence>").unwrap();
        println!("[CLI] >>> {}", str::from_utf8(writer.buffer()).unwrap());
        writer.flush().unwrap();

        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

        // End stream
        let end_stream = crate::packet::PacketExporter::export(Packet::End).unwrap();
        writer.write(end_stream.as_bytes()).unwrap();
        writer.flush().unwrap();
        println!("[CLI] >>> {}", end_stream);
        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());

    }
}