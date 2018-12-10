mod simplenet;

use crate::simplenet::*;
use custom_error::custom_error;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::net::IpAddr;

custom_error! {MyError
    Pcap{source: pcap::Error} = "Pcap error",
}

fn main() -> Result<(), MyError> {
    let mut cap = Capture::from_file("/home/eric/Downloads/test.pcap")?;
    let mut packets = Vec::new();
    while let Ok(p) = cap.next() {
        if let Some(ether_packet) = EthernetPacket::new(&p) {
            let pack = handle_ethernet_frame(&ether_packet, &p.header.ts);
            packets.push(pack);
        }
    }
    let p: Vec<&SimplePacket> = packets
        .iter()
        .filter_map(|p| match p {
            Some(x) => Some(x),
            _ => None,
        })
        .collect();
    println!("{:#?}", p);
    let mut p: Vec<&IpAddr> = packets
        .iter()
        .filter_map(|p| match p {
            Some(SimplePacket::Ip(x)) => Some(&x.sender_proto_addr),
            _ => None,
        })
        .collect();
    p.sort();
    p.dedup();
    println!("{:#?}", p);
    Ok(())
}
