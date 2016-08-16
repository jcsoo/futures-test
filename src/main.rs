#![allow(unused_imports, dead_code)]

extern crate futures;
extern crate futures_io;
extern crate futures_mio;
extern crate trust_dns;

mod dns_query;

use trust_dns::udp::UdpClientConnection;
use trust_dns::rr::record_data::RData;

use futures::*;
use futures::stream::Stream;
use futures_mio::Loop;

use std::net::SocketAddr;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

fn main() {
    let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    let buf = dns_query::build_query(1234, "google.com");


    let mut lp = futures_mio::Loop::new().unwrap();
    let local_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let socket_bind = lp.handle().udp_bind(&local_addr); // Future<Item = UdpSocket, Error = Error>

    let done = socket_bind.and_then(move |socket| {
        //let socket_future = socket.into_future(); // Future<Item = (Option<Ready>, UdpSocket)>, Error = (Error, UdpSocket)>
        socket
            .into_future()
            .and_then(move |(ar, a)| {
                let ar = ar.unwrap(); // Ready
                assert!(ar.is_write());

                a.send_to(&buf, &addr).unwrap();
                a.into_future() // Future<Item = (Option<Ready>, UdpSocket)>
            })
            .and_then(|(ar,a)| {
                let ar = ar.unwrap(); // Ready
                assert!(ar.is_read());

                let mut buf = [0; 512];
                let (_size, _addr) = a.recv_from(&mut buf).unwrap();

                Ok(dns_query::parse_response(&mut buf))
            })
            .map_err(|(e, _)| e)
    });
    let msg = lp.run(done).unwrap(); // (Option<Ready>, UdpSocket)
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
