use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

use std::str;

use native_tls::{TlsConnector, TlsStream};
use xmpp_parsers::ns;
use xmpp_parsers::Element;

use crate::{
    packet::{Packet, PacketExporter, PacketImporter},
    user::User,
};

const STREAM_VERSION: &str = "1.0";
// TODO: Switch to xmpp_parsers::ns
const XML_NAMESPACE: &str = "jabber:client";
const XML_NAMESPACE_STREAM: &str = "http://etherx.jabber.org/streams";

pub struct XMPPStream {
    // stream: Rc<StreamTypes>,
    connection: String,
    stream: Option<TlsStream<TcpStream>>,
    // tls_stream: Rc<Option<TlsStream<TcpStream>>>,
    user: User,
    started: bool,
    pub header_attrs: HashMap<String, String>,
    pub stream_features: Option<Element>,
}

impl XMPPStream {
    pub fn new(connection: String, user: User) -> Self {
        Self {
            // stream: Rc::new(stream),
            connection,
            stream: None,
            user,
            started: false,
            header_attrs: HashMap::new(),
            stream_features: None,
        }
    }

    fn get_reader(&mut self) -> BufReader<&mut TlsStream<TcpStream>> {
        let stream = self.stream.as_mut().unwrap();
        let reader = BufReader::new(stream);

        reader
    }

    fn get_writer(&mut self) -> BufWriter<&mut TlsStream<TcpStream>> {
        let stream = self.stream.as_mut().unwrap();
        let reader = BufWriter::new(stream);

        reader
    }

    pub fn send(&mut self, packet: Packet) {
        let mut writer = self.get_writer();

        let content = PacketExporter::export(packet).unwrap();

        println!("[TLS CLI] >>> {}", content);

        writer.write(content.as_bytes()).unwrap();
        writer.flush().unwrap();
    }

    fn send_stream(&self, stream: &TcpStream, packet: Packet) {
        let mut writer = BufWriter::new(stream);

        let content = PacketExporter::export(packet).unwrap();

        println!("[CLI] >>> {}", content);

        writer.write(content.as_bytes()).unwrap();
        writer.flush().unwrap();
    }

    pub fn read(&mut self) -> String {
        let mut reader = self.get_reader();

        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());

        let received_str = str::from_utf8(&received).unwrap();

        println!("[TLS SER] <<< {}", received_str);

        received_str.to_owned()
    }

    fn read_stream(&self, stream: &TcpStream) -> String {
        let mut reader = BufReader::new(stream);

        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());

        let received_str = str::from_utf8(&received).unwrap();

        println!("[SER] <<< {}", received_str);

        received_str.to_owned()
    }

    fn is_valid_header(&mut self, header: Packet) -> bool {
        if !self.header_attrs.is_empty() {
            return true;
        }

        if let Packet::Start(attrs) = header {
            if attrs.get("from").unwrap() != &self.user.username.clone().domain() {
                return false;
            }

            self.header_attrs = attrs;
            return true;
        }

        false
    }

    pub fn start_stream(&mut self) {
        let tcp_stream = TcpStream::connect(self.connection.clone()).unwrap();
        // let mut reader = BufReader::new(tcp_stream.as_ref());

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

        self.send_stream(&tcp_stream, stream_start_packet.clone());

        let received = self.read_stream(&tcp_stream);

        let header = PacketImporter::import(received).unwrap();

        if !self.is_valid_header(header) {
            panic!("Server send invalid header");
        }

        let received = self.read_stream(&tcp_stream);

        let res = PacketImporter::import(received).unwrap();
        if let Packet::Stanza(element) = res {
            self.stream_features = Some(element);
        } else {
            panic!("Features packet wasn't of type stanza. Packet: {:?}.", res)
        }

        let tls_stream = self.start_tls(tcp_stream); // This panics

        // RFC 6120 section 5.4.3.3 states we need to start a new stream and discard any info we got before.
        // Basically, we need to redo everything we did above but with the new stream.
        self.header_attrs = HashMap::new();
        self.stream_features = None;
        self.stream = Some(tls_stream);

        self.send(stream_start_packet);
        let header = PacketImporter::import(self.read()).unwrap();

        if !self.is_valid_header(header) {
            panic!("Server sent invalid header.");
        }

        // TODO: Since we started a new stream due to enabling TLS, we need to get the stanza's from the received stream header.
        // This is old code when we weren't using starttls. Should be able to get the child of the header packet.
        // let res = PacketImporter::import(received).unwrap();
        // if let Packet::Stanza(element) = res {
        //     self.stream_features = Some(element);
        // } else { panic!("Features packet wasn't of type stanza. Packet: {:?}.", res) }

        self.started = true;
    }

    pub fn start_tls(&mut self, stream: TcpStream) -> TlsStream<TcpStream> {
        if let Some(features) = self.stream_features.clone() {
            if features.has_child("starttls", "urn:ietf:params:xml:ns:xmpp-tls") {
                self.send_stream(&stream, Packet::Stanza(Element::bare("starttls", ns::TLS)));
                println!("[CLI] >>> <starttls/>");

                let confirmation: Element = self.read_stream(&stream).parse().unwrap();

                if confirmation.name() == "proceed" {
                    let connector = TlsConnector::new().unwrap();
                    return connector
                        .connect(&self.user.username.clone().domain(), stream)
                        .unwrap();
                }

                panic!("Could not initialize TCP connection.");
            } else {
                panic!("Server does not support TCP.");
            }
        }

        panic!("Server did not send any stream features.");
    }
}
