use std::collections::HashMap;

type PacketExportResult = Result<String, PacketExportError>;

#[derive(Clone)]
pub enum Packet {
    Start(HashMap<String, String>),
    End,
}

#[derive(Debug)]
pub enum PacketExportError {
    InvalidPacketType
}

pub struct PacketExporter;

impl PacketExporter {
    pub fn export(packet: Packet) -> PacketExportResult {
        match packet.clone() {
            Packet::Start(_) => PacketExporter::create_start_packet(packet),
            Packet::End => Ok(String::from("</stream:stream>")),
        }
    }

    fn create_start_packet(packet: Packet) -> PacketExportResult {
        if let Packet::Start(map) = packet {
            let mut result = String::from("<?xml version='1.0'?><stream:stream ");
            result.push_str("");
            for (i, (key, value)) in map.iter().enumerate() {
                result.push_str(&format!("{key}='{value}'"));
                if i < map.len() - 1{
                    result.push_str(" ");
                }
            }

            result.push_str(">");

            return Ok(result);
        }

        Err(PacketExportError::InvalidPacketType)
    }
}
