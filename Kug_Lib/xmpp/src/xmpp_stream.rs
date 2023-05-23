use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    rc::Rc,
};

use std::str;

use crate::{
    packet::{Packet, PacketExporter},
    user::User,
};

const STREAM_VERSION: &str = "1.0";
const XML_NAMESPACE: &str = "jabber:client";
const XML_NAMESPACE_STREAM: &str = "http://etherx.jabber.org/streams";

pub struct XMPPStream {
    stream: Rc<TcpStream>,
    user: User,
}

impl XMPPStream {
    pub fn new(stream: TcpStream, user: User) -> Self {
        Self {
            stream: Rc::new(stream),
            user,
        }
    }

    pub fn send(&self, packet: Packet) {
        let stream = Rc::clone(&self.stream);
        let mut writer = BufWriter::new(stream.as_ref());

        let content = PacketExporter::export(packet).unwrap();

        println!("[CLI] >>> {}", content);

        writer.write(content.as_bytes()).unwrap();
        writer.flush().unwrap();
    }

    pub fn read(&self) -> String {
        let stream = Rc::clone(&self.stream);
        let mut reader = BufReader::new(stream.as_ref());

        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());

        let received_str = str::from_utf8(&received).unwrap();

        println!("[SER] <<< {}", received_str);

        received_str.to_owned()
    }

    pub fn start_stream(&self) {
        let stream = Rc::clone(&self.stream);
        let mut reader = BufReader::new(stream.as_ref());

        let stream_start_packet = Packet::Start(HashMap::from([
            (String::from("from"), self.user.username.to_string()),
            (String::from("to"), self.user.username.clone().domain()),
            (String::from("version"), String::from(STREAM_VERSION)),
            (String::from("xmlns"), String::from(XML_NAMESPACE)),
            (
                String::from("xmlns:stream"),
                String::from(XML_NAMESPACE_STREAM),
            ),
        ]));

        self.send(stream_start_packet);

        // TODO: Read the header and make sure it's valid, instead of discarding it.
        // Example: Wrong 'from' field value (not the one we sent)
        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        println!("[SER] <<< {}", str::from_utf8(received.as_slice()).unwrap());
    }
}
