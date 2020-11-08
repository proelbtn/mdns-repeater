extern crate anyhow;
#[macro_use]
extern crate clap;
extern crate log;
extern crate nix;
extern crate serde;

use std::env;
use std::ffi::OsString;
use std::os::unix::io::RawFd;

use anyhow::Result;
use clap::{App, Arg};
use log::{info, trace};
use nix::sys::epoll::*;
use nix::sys::socket::*;
use nix::sys::socket::sockopt::{BindToDevice, IpAddMembership, IpMulticastLoop};
use serde::{Serialize, Deserialize};

const ARGS_CONFIG_FILE: &'static str = "CONFIG";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct InterfaceConfig {
    name: String,
    address: String,
    netmask: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GlobalConfig {
    interfaces: Vec<InterfaceConfig>,
}

#[derive(Debug)]
struct Interface {
    name: String,
    address: std::net::IpAddr,
    netmask: std::net::IpAddr,
    sockfd: RawFd,
}

impl Interface {
    fn new(config: &InterfaceConfig) -> Result<Self> {
        let address: std::net::IpAddr = config.address.parse()?;
        let netmask: std::net::IpAddr = config.netmask.parse()?;

        let family = AddressFamily::Inet;
        let socktype = SockType::Datagram;
        let flag = SockFlag::empty();
        let proto = SockProtocol::Udp;

        let fd = socket(family, socktype, flag, proto)?;

        setsockopt(fd, BindToDevice, &OsString::from(&config.name))?;

        Ok(Interface {
            name: config.name.clone(),
            address: address,
            netmask: netmask,
            sockfd: fd,
        })
    }

    fn has(&self, addr: std::net::IpAddr) -> bool {
        return self.address == addr
    }
}

fn is_same_network_v4(
    addr: std::net::Ipv4Addr,
    ifaddr: std::net::Ipv4Addr,
    mask: std::net::Ipv4Addr) -> bool {
    for i in 0..3 {
        if addr.octets()[i] & mask.octets()[i] 
            != ifaddr.octets()[i] & mask.octets()[i] {
            return false;
        }
    };
    return true;
}

fn is_same_network_v6(
    addr: std::net::Ipv6Addr,
    ifaddr: std::net::Ipv6Addr,
    mask: std::net::Ipv6Addr) -> bool {
    for i in 0..15 {
        if addr.octets()[i] & mask.octets()[i] 
            != ifaddr.octets()[i] & mask.octets()[i] {
            return false;
        }
    };
    return true;
}

fn is_same_network(interface: &Interface, address: std::net::IpAddr)-> bool {
    match interface.address {
        std::net::IpAddr::V4(ifaddr) => match interface.netmask {
            std::net::IpAddr::V4(mask) => match address {
                std::net::IpAddr::V4(addr) =>
                    return is_same_network_v4(addr, ifaddr, mask),
                _ => return false
            },
            _ => return false
        },
        std::net::IpAddr::V6(ifaddr) => match interface.netmask {
            std::net::IpAddr::V6(mask) => match address {
                std::net::IpAddr::V6(addr) =>
                    return is_same_network_v6(addr, ifaddr, mask),
                _ => return false
            },
            _ => return false
        },
    }
}

fn setup_app<'a, 'b>() -> App<'a, 'b> {
    let name = env!("CARGO_PKG_NAME");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let version = crate_version!();
    let author = env!("CARGO_PKG_AUTHORS");
    App::new(name)
        .version(version)
        .author(author)
        .about(description)
        .arg(Arg::with_name(ARGS_CONFIG_FILE)
            .short("f")
            .long("config-file")
            .default_value("config.yaml")
            .help("Config file")
            .required(true))
}

fn load_config(config_file: &str) -> Result<Box<GlobalConfig>> {
    let config = std::fs::read_to_string(config_file)?;
    let config: GlobalConfig = serde_yaml::from_str(&config)?;
    Ok(Box::new(config))
}

fn setup_receive_socket(config: Box<GlobalConfig>) -> Result<RawFd> {
    let family = AddressFamily::Inet;
    let socktype = SockType::Datagram;
    let flag = SockFlag::empty();
    let proto = SockProtocol::Udp;

    let fd = socket(family, socktype, flag, proto)?;

    let recvaddr = SockAddr::new_inet(
        InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 5353));
    bind(fd, &recvaddr)?;

    setsockopt(fd, IpMulticastLoop, &true)?;

    for interface in config.interfaces {
        let maddr = Ipv4Addr::new(224, 0, 0, 251);
        let ifaddr = Ipv4Addr::from_std(&interface.address.parse()?);
        let ip_mreq = IpMembershipRequest::new(maddr, Some(ifaddr));
        setsockopt(fd, IpAddMembership, &ip_mreq)?;
    }

    Ok(fd)
}

fn start(config: Box<GlobalConfig>) -> Result<()> {
    info!("Setting up interfaces...");
    let mut interfaces = Vec::new();
    for interface in &config.interfaces {
        let interface = Interface::new(interface)?;
        trace!("Added interface: {:?}", interface);
        interfaces.push(interface);
    }

    info!("Setting up receive socket...");
    let recvfd = setup_receive_socket(config)?;

    info!("Setting up epoll...");
    let epoll_fd = epoll_create()?;
    let mut epoll_events = vec![EpollEvent::empty(); 16];
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN, recvfd as u64);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, recvfd, &mut event)?;

    info!("Starting poll...");
    let dst = SockAddr::new_inet(
        InetAddr::new(IpAddr::new_v4(224, 0, 0, 251), 5353));
    loop {
        let num = epoll_wait(epoll_fd, &mut epoll_events, 100)?;

        'events: for i in 0..num {
            let mut buf: [u8; 4096] = [0; 4096];

            let sockfd = epoll_events[i].data() as RawFd;
            let (len, addr) = recvfrom(sockfd, &mut buf)?;

            if let Some(SockAddr::Inet(addr)) = addr {
                let addr = addr.ip().to_std();
                for interface in &interfaces {
                    if interface.has(addr) {
                        continue 'events;
                    }
                }

                trace!("Received mdns packets from {:?} (sockfd: {})", addr, sockfd);
                for interface in &interfaces {
                    if !is_same_network(interface, addr) {
                        trace!("Sending Multicast DNS packets to {}", interface.sockfd);
                        sendto(interface.sockfd, &buf[0..len], &dst, MsgFlags::empty())?;
                    }
                }
            }
        }
    };
    Ok(())
}

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    // parse config and load config
    let matches = setup_app().get_matches();
    let config_file: &str = matches.value_of(ARGS_CONFIG_FILE).unwrap();
    let config = load_config(config_file)?;

    // start 
    start(config)?;

    Ok(())
}
