#![allow(unused_imports, unused_variables, dead_code)]

extern crate futures;
extern crate futures_io;
extern crate futures_mio;
extern crate trust_dns;

mod dns_query;

use dns_query::Message;

use futures::{Future, Poll};
use futures_mio::{Loop, UdpSocket};

use std::net::SocketAddr;
use std::io;

#[derive(Debug)]
enum ResolveState {
    Send,
    Recv,
}

struct ResolveDns {
    socket: UdpSocket,
    remote: SocketAddr,
    buffer: Vec<u8>,
    state: ResolveState,    
}

impl ResolveDns {
    pub fn new(socket: UdpSocket, remote: SocketAddr, message: Message) -> Self {
        let mut buf: Vec<u8> = Vec::with_capacity(2048);        
        dns_query::encode_message(&mut buf, &message);
        ResolveDns { socket: socket, remote: remote, buffer: buf, state: ResolveState::Send }
    }
}

impl Future for ResolveDns {
    type Item = Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {        
        loop {            
            match self.state {
                ResolveState::Send => {
                    match self.socket.poll_write() {
                        Poll::Ok(_) => {                        
                            self.socket.send_to(&self.buffer, &self.remote).unwrap();
                            self.state = ResolveState::Recv;                        
                        },
                        _ => return Poll::NotReady,
                    }
                    
                },
                ResolveState::Recv => {
                    match self.socket.poll_read() {
                        Poll::Ok(_) => {                        
                            let mut buf = [0u8; 2048];
                            let _  = self.socket.recv_from(&mut buf).unwrap();
                            let msg = dns_query::decode_message(&mut buf);
                            return Poll::Ok(msg);
                        },
                        _ => return Poll::NotReady,
                    }
                }
            }
        }
    }
}



fn main() {
    let mut lp = Loop::new().unwrap();
        
    let remote_addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    let msg = dns_query::build_query_message(dns_query::a_query("google.com"));

    let socket_bind = lp.handle().udp_bind(&"0.0.0.0:0".parse().unwrap()); // Future<Item = UdpSocket, Error = Error>
    
    let done = socket_bind
        .and_then(move |socket| ResolveDns::new(socket, remote_addr, msg));
    
    
    let v = lp.run(done).unwrap();
    println!("v: {:?}", v);
}