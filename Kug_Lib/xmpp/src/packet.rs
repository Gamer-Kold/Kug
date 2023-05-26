use std::collections::HashMap;
use std::str;
use xml::{
    attribute::OwnedAttribute,
    reader::{EventReader, XmlEvent},
};
use xmpp_parsers::Element;

type PacketExportResult = Result<String, PacketError>;
type PacketImportResult = Result<Packet, PacketError>;

#[derive(Clone, Debug)]
pub enum Packet {
    Start(HashMap<String, String>),
    Stanza(Element),
    End,
}

#[derive(Debug)]
pub enum PacketError {
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

        Err(PacketError::InvalidPacketType)
    }
}

pub struct PacketImporter;

impl PacketImporter {
    fn convert_attributes(attributes: Vec<OwnedAttribute>) -> HashMap<String, String> {
        let mut converted_attributes: HashMap<String, String> = HashMap::new();

        for attribute in attributes {
            converted_attributes.insert(attribute.name.to_string(), attribute.value);
        }

        converted_attributes
    }

    pub fn import(content: String) -> PacketImportResult {
        // TODO: There HAS to be a better way to do this. XML parser does not like that there is no namespace definition for :features.
        let new_content = content.replace(
            "<stream:features>",
            "<stream:features xmlns:stream='http://etherx.jabber.org/streams'>",
        );
        let mut parser = EventReader::new(new_content.as_bytes());
        parser.next().unwrap(); // Skip XmlEvent::StartDocument
        let parsed_root = parser.next().unwrap();

        if let XmlEvent::StartElement {
            name, attributes, ..
        } = parsed_root
        {
            let converted_attributes = PacketImporter::convert_attributes(attributes);

            if name.prefix.is_some() {
                match name.local_name.as_str() {
                    "stream" => return Ok(Packet::Start(converted_attributes)),
                    "features" => return Ok(Packet::Stanza(new_content.parse().unwrap())),
                    _ => panic!("Unknown content with prefix"),
                }
            }

            match name.local_name.as_str() {
                "presence" => todo!("presence"),
                "message" => todo!("message"),
                "iq" => Ok(Packet::Stanza(
                    Element::from_reader_with_prefixes(
                        content.as_bytes(),
                        String::from("jabber:client"),
                    )
                    .unwrap(),
                )),
                _ => todo!(),
            }
        } else {
            todo!("XML was {:?}", parsed_root);
        }
    }
}
