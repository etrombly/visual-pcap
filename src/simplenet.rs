use chrono::naive::NaiveDateTime;
use libc::timeval;
use pnet::packet::{
    arp::{ArpOperation, ArpPacket},
    ethernet::{EtherTypes, EthernetPacket},
    icmp::{IcmpCode, IcmpPacket, IcmpType},
    icmpv6::{Icmpv6Code, Icmpv6Packet, Icmpv6Type},
    ip::{IpNextHeaderProtocol, IpNextHeaderProtocols},
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    tcp::TcpPacket,
    udp::UdpPacket,
    Packet,
};
use pnet::util::MacAddr;
use std::net::IpAddr;

#[derive(Debug)]
pub enum SimplePacket {
    Ip(SimpleIp),
    Arp(SimpleArp),
}

#[derive(Debug)]
pub enum SimpleProto {
    Udp(SimpleUdp),
    Tcp(SimpleTcp),
    Icmp(SimpleIcmp),
    Icmpv6(SimpleIcmpv6),
}

#[derive(Debug)]
pub struct SimpleArp {
    pub operation: ArpOperation,
    pub sender_hw_addr: MacAddr,
    pub sender_proto_addr: IpAddr,
    pub target_hw_addr: MacAddr,
    pub target_proto_addr: IpAddr,
    pub ts: NaiveDateTime,
}

impl SimpleArp {
    pub fn new(
        operation: ArpOperation,
        sender_hw_addr: MacAddr,
        sender_proto_addr: IpAddr,
        target_hw_addr: MacAddr,
        target_proto_addr: IpAddr,
        ts: NaiveDateTime,
    ) -> SimpleArp {
        SimpleArp {
            operation,
            sender_hw_addr,
            sender_proto_addr,
            target_hw_addr,
            target_proto_addr,
            ts,
        }
    }
}

#[derive(Debug)]
pub struct SimpleIp {
    pub sender_hw_addr: MacAddr,
    pub sender_proto_addr: IpAddr,
    pub target_hw_addr: MacAddr,
    pub target_proto_addr: IpAddr,
    pub proto: Option<SimpleProto>,
    pub ts: NaiveDateTime,
}

impl SimpleIp {
    pub fn new(
        sender_hw_addr: MacAddr,
        sender_proto_addr: IpAddr,
        target_hw_addr: MacAddr,
        target_proto_addr: IpAddr,
        proto: Option<SimpleProto>,
        ts: NaiveDateTime,
    ) -> SimpleIp {
        SimpleIp {
            sender_hw_addr,
            sender_proto_addr,
            target_hw_addr,
            target_proto_addr,
            proto,
            ts,
        }
    }
}

#[derive(Debug)]
pub struct SimpleUdp {
    pub source: u16,
    pub destination: u16,
    pub size: u16,
}

#[derive(Debug)]
pub struct SimpleTcp {
    pub source: u16,
    pub destination: u16,
    pub size: u16,
}

#[derive(Debug)]
pub struct SimpleIcmp {
    pub icmp_type: IcmpType,
    pub icmp_code: IcmpCode,
}

#[derive(Debug)]
pub struct SimpleIcmpv6 {
    pub icmp_type: Icmpv6Type,
    pub icmp_code: Icmpv6Code,
}

pub fn handle_udp_packet(packet: &[u8]) -> Option<SimpleProto> {
    let udp = UdpPacket::new(packet);

    if let Some(udp) = udp {
        Some(SimpleProto::Udp(SimpleUdp {
            source: udp.get_source(),
            destination: udp.get_destination(),
            size: udp.get_length(),
        }))
    } else {
        None
    }
}

pub fn handle_icmp_packet(packet: &[u8]) -> Option<SimpleProto> {
    let icmp_packet = IcmpPacket::new(packet);
    if let Some(icmp_packet) = icmp_packet {
        Some(SimpleProto::Icmp(SimpleIcmp {
            icmp_type: icmp_packet.get_icmp_type(),
            icmp_code: icmp_packet.get_icmp_code(),
        }))
    } else {
        None
    }
}

pub fn handle_icmpv6_packet(packet: &[u8]) -> Option<SimpleProto> {
    let icmpv6_packet = Icmpv6Packet::new(packet);
    if let Some(icmpv6_packet) = icmpv6_packet {
        Some(SimpleProto::Icmpv6(SimpleIcmpv6 {
            icmp_type: icmpv6_packet.get_icmpv6_type(),
            icmp_code: icmpv6_packet.get_icmpv6_code(),
        }))
    } else {
        None
    }
}

pub fn handle_tcp_packet(packet: &[u8]) -> Option<SimpleProto> {
    let tcp = TcpPacket::new(packet);
    if let Some(tcp) = tcp {
        Some(SimpleProto::Tcp(SimpleTcp {
            source: tcp.get_source(),
            destination: tcp.get_destination(),
            size: packet.len() as u16,
        }))
    } else {
        None
    }
}

pub fn handle_transport_protocol(
    protocol: IpNextHeaderProtocol,
    packet: &[u8],
) -> Option<SimpleProto> {
    match protocol {
        IpNextHeaderProtocols::Udp => handle_udp_packet(packet),
        IpNextHeaderProtocols::Tcp => handle_tcp_packet(packet),
        IpNextHeaderProtocols::Icmp => handle_icmp_packet(packet),
        IpNextHeaderProtocols::Icmpv6 => handle_icmpv6_packet(packet),
        _ => None,
    }
}

pub fn handle_ipv4_packet(ethernet: &EthernetPacket, ts: &timeval) -> Option<SimplePacket> {
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        let proto = handle_transport_protocol(header.get_next_level_protocol(), header.payload());
        Some(SimplePacket::Ip(SimpleIp::new(
            ethernet.get_source(),
            IpAddr::V4(header.get_source()),
            ethernet.get_destination(),
            IpAddr::V4(header.get_destination()),
            proto,
            NaiveDateTime::from_timestamp(ts.tv_sec, ts.tv_usec as u32),
        )))
    } else {
        None
    }
}

pub fn handle_ipv6_packet(ethernet: &EthernetPacket, ts: &timeval) -> Option<SimplePacket> {
    let header = Ipv6Packet::new(ethernet.payload());
    if let Some(header) = header {
        let proto = handle_transport_protocol(header.get_next_header(), header.payload());
        Some(SimplePacket::Ip(SimpleIp::new(
            ethernet.get_source(),
            IpAddr::V6(header.get_source()),
            ethernet.get_destination(),
            IpAddr::V6(header.get_destination()),
            proto,
            NaiveDateTime::from_timestamp(ts.tv_sec, ts.tv_usec as u32),
        )))
    } else {
        None
    }
}

pub fn handle_arp_packet(ethernet: &EthernetPacket, ts: &timeval) -> Option<SimplePacket> {
    let header = ArpPacket::new(ethernet.payload());
    if let Some(header) = header {
        Some(SimplePacket::Arp(SimpleArp::new(
            header.get_operation(),
            ethernet.get_source(),
            IpAddr::V4(header.get_sender_proto_addr()),
            ethernet.get_destination(),
            IpAddr::V4(header.get_target_proto_addr()),
            NaiveDateTime::from_timestamp(ts.tv_sec, ts.tv_usec as u32),
        )))
    } else {
        None
    }
}

pub fn handle_ethernet_frame(ethernet: &EthernetPacket, ts: &timeval) -> Option<SimplePacket> {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(ethernet, ts),
        EtherTypes::Ipv6 => handle_ipv6_packet(ethernet, ts),
        EtherTypes::Arp => handle_arp_packet(ethernet, ts),
        _ => None,
    }
}
