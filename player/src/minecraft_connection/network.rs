use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use minecraft_protocol::{packets as mc_packets, MinecraftPacketPart};

pub fn send_packet<'a>(
    stream: &mut TcpStream,
    packet: impl MinecraftPacketPart<'a>,
) -> io::Result<()> {
    let packet = packet.serialize_minecraft_packet().unwrap();
    send_packet_raw(stream, packet.as_slice())?;
    Ok(())
}

pub fn send_packet_raw(stream: &mut TcpStream, packet: &[u8]) -> io::Result<()> {
    let length = mc_packets::VarInt::from(packet.len());

    let length_packet = length.serialize_minecraft_packet()
        .map_err(|s| io::Error::new(io::ErrorKind::InvalidInput, s))?;

    stream.write_all(length_packet.as_slice())?;
    stream.write_all(packet)?;
    stream.flush()?;
    Ok(())
}

pub fn receive_packet<'a, PacketType>(stream: &mut TcpStream, buffer: &'a mut Vec<u8>) -> io::Result<PacketType>
where
    PacketType: MinecraftPacketPart<'a>,
{
    *buffer = receive_packet_raw(stream)?;
    PacketType::deserialize_uncompressed_minecraft_packet(buffer.as_slice())
        .map_err(|s| io::Error::new(io::ErrorKind::InvalidData, s))
}

pub fn receive_packet_raw(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut length: Vec<u8> = Vec::with_capacity(2);

    loop {
        if length.len() >= 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid length field",
            ));
        }
        let mut byte = [0];
        stream.read_exact(&mut byte)?;
        length.push(byte[0]);
        if byte[0] < 0b1000_0000 {
            break;
        }
    }

    let length = mc_packets::VarInt::deserialize_uncompressed_minecraft_packet(length.as_mut_slice())
        .map_err(|s| io::Error::new(io::ErrorKind::InvalidData, s))?;

    let mut data = Vec::with_capacity(length.0 as usize);
    unsafe {
        data.set_len(length.0 as usize);
    }
    stream.read_exact(&mut data)?;

    Ok(data)
}
