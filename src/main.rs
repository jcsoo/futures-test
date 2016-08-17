#![allow(unused_imports, dead_code)]

extern crate futures;
extern crate futures_io;
extern crate futures_mio;
extern crate trust_dns;

mod dns_query;

use trust_dns::udp::UdpClientConnection;
use trust_dns::rr::record_data::RData;
use trust_dns::op::Message;

use futures::*;
use futures::stream::Stream;
use futures_io::IoFuture;
use futures_mio::{Loop, LoopHandle, UdpSocket};

use std::net::SocketAddr;
use std::io;

#[derive(Debug)]
pub struct Resolver {
    socket: UdpSocket,
    addr: SocketAddr,
}

impl Resolver {
    pub fn new(lh: LoopHandle, addr: SocketAddr) -> BoxFuture<Resolver, io::Error> {
        let local_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
        let socket_bind = lh.udp_bind(&local_addr); // Future<Item = UdpSocket, Error = Error>
        socket_bind.map(move |socket| { 
            Resolver {
                socket: socket,
                addr: addr
            }
        }).boxed()
    }

    pub fn resolve(self, host: &str) -> BoxFuture<(Message, Resolver), (io::Error, Resolver)> {
        let buf = dns_query::build_query(1234, host);
        let addr = self.addr;        
        
        self.socket
            .into_future()
            .and_then(move |(ar, a)| {
                let ar = ar.unwrap(); // Ready
                assert!(ar.is_write());

                a.send_to(&buf, &addr).unwrap();
                a.into_future() // Future<Item = (Option<Ready>, UdpSocket)>
            })
            .and_then(move |(ar,a)| {
                let ar = ar.unwrap(); // Ready
                assert!(ar.is_read());

                let mut buf = [0; 512];
                let (_size, _addr) = a.recv_from(&mut buf).unwrap();

                let r = Resolver { socket: a, addr: addr};
                Ok((dns_query::parse_response(&mut buf), r))
            })
            .map_err(move |(e, a)| {
                let r = Resolver { socket: a, addr: addr};
                (e, r)
            }).boxed()
       }
}

fn main() {
    let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    let host = "google.com";

    let mut lp = futures_mio::Loop::new().unwrap();
    let h = lp.handle();
    let r = lp.run(Resolver::new(h, addr)).unwrap();
    let (msg, _) = lp.run(r.resolve(host)).unwrap();
    println!("msg: {:?}", msg);
}

fn main_basic() {
    let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    let buf = dns_query::build_query(1234, "google.com");


    let mut lp = futures_mio::Loop::new().unwrap();
    let local_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let socket_bind = lp.handle().udp_bind(&local_addr); // IoFuture<UdpSocket>
    let socket = lp.run(socket_bind).unwrap(); // UdpSocket
    let socket_future = socket.into_future(); // Future<Item = (Option<Ready>, UdpSocket)>>
    let (ar, a) = lp.run(socket_future).unwrap(); // (Option<Ready>, UdpSocket)
    let ar = ar.unwrap(); // Ready
    assert!(ar.is_write());

    a.send_to(&buf, &addr).unwrap();
    let a_future = a.into_future(); // Future<Item = (Option<Ready>, UdpSocket)>
    let (ar, a) = lp.run(a_future).unwrap(); // (Option<Ready>, UdpSocket)
    let ar = ar.unwrap(); // Ready
    assert!(ar.is_read());

    let mut buf = [0; 512];
    let (_size, _addr) = a.recv_from(&mut buf).unwrap();

    let msg = dns_query::parse_response(&mut buf);
    println!("msg: {:?}", msg);
}
