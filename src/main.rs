#[macro_use]
extern crate clap;
extern crate log;
extern crate nix;

use std::env;
use std::ffi::OsString;
use std::os::unix::io::RawFd;

use clap::{App, Arg};
use log::info;
use nix::sys::epoll::*;
use nix::sys::socket::*;
use nix::sys::socket::sockopt::BindToDevice;

const ARGS_INTERFACES: &'static str = "INTERFACES";

#[derive(Debug)]
struct SocketPair {
    recv: RawFd,
    send: RawFd,
}

#[derive(Debug)]
struct Interface {
    name: String,
    sockpair: SocketPair,
}

impl Interface {
    fn new(name: &str) -> nix::Result<Self> {
        let family = AddressFamily::Inet;
        let socktype = SockType::Datagram;
        let flag = SockFlag::empty();
        let proto = SockProtocol::Udp;

        let recvfd = socket(family, socktype, flag, proto)?;
        let sendfd = socket(family, socktype, flag, proto)?;

        setsockopt(recvfd, BindToDevice, &OsString::from(name))?;
        setsockopt(sendfd, BindToDevice, &OsString::from(name))?;

        let recvaddr = SockAddr::new_inet(
            InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 5353));
        bind(recvfd, &recvaddr)?;

        Ok(Interface {
            name: String::from(name),
            sockpair: SocketPair {
                recv: recvfd,
                send: sendfd,
            }
        })
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
        .arg(Arg::with_name(ARGS_INTERFACES)
            .help("Interface names where mdns-repeater works")
            .required(true)
            .multiple(true))
}

fn start() -> nix::Result<()> {
    let matches = setup_app().get_matches();
    let ifnames = matches.values_of(ARGS_INTERFACES).unwrap();

    info!("Setting up interfaces...");
    let mut interfaces = Vec::new();
    for ifname in ifnames {
        let interface = Interface::new(ifname)?;
        interfaces.push(interface);
    }

    let epoll_fd = epoll_create()?;
    let mut epoll_events = vec![EpollEvent::empty(); 1024];

    for interface in &interfaces {
        let sockfd = interface.sockpair.recv;
        let mut event = EpollEvent::new(EpollFlags::EPOLLIN, sockfd as u64);
        epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, sockfd, &mut event)?;
    }

    info!("Starting poll...");
    loop {
        println!("test");
        let num = epoll_wait(epoll_fd, &mut epoll_events, -1)?;

        for i in 0..num {
            let mut buf: [u8; 4096] = [0; 4096];

            let sockfd = epoll_events[i].data() as RawFd;
            recv(sockfd, &mut buf, MsgFlags::empty())?;

            for interface in &interfaces {
                if interface.sockpair.recv != sockfd {
                    send(interface.sockpair.send, &buf, MsgFlags::empty())?;
                }
            }
        }
    }
}

fn main() {
    env::set_var("RUST_LOG", env!("RUST_LOG"));
    env_logger::init();
    start().unwrap();
}
