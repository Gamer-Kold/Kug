use std::collections::HashMap;
use std::str;
use xmpp_parsers::Element;

type PacketExportResult = Result<String, PacketExportError>;

#[derive(Clone)]
pub enum Packet {
    Start(HashMap<String, String>),
    Stanza(Element),
    End,
}

#[derive(Debug)]
pub enum PacketExportError {
    InvalidPacketType,
}

pub struct PacketExporter;

impl PacketExporter {
    pub fn export(packet: Packet) -> PacketExportResult {
        match packet.clone() {
            Packet::Start(_) => PacketExporter::create_start_packet(packet),
            Packet::Stanza(element) => PacketExporter::create_stanza(element),
            Packet::End => Ok(String::from("</stream:stream>")),
        }
    }

    fn create_stanza(element: Element) -> PacketExportResult {
        let mut result: Vec<u8> = Vec::new();
        element.write_to(&mut result).unwrap();

        Ok(str::from_utf8(&result).unwrap().to_owned())
    }

    fn create_start_packet(packet: Packet) -> PacketExportResult {
        if let Packet::Start(map) = packet {
            let mut result = String::from("<?xml version='1.0'?><stream:stream ");
            result.push_str("");
            for (i, (key, value)) in map.iter().enumerate() {
                result.push_str(&format!("{key}='{value}'"));
                if i < map.len() - 1 {
                    result.push_str(" ");
                }
            }

            result.push_str(">");

            return Ok(result);
        }

        Err(PacketExportError::InvalidPacketType)
    }
}
