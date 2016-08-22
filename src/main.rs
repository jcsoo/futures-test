#![allow(unused_imports, unused_variables, dead_code)]

extern crate futures;
#[macro_use]
extern crate futures_io;
extern crate futures_mio;
extern crate trust_dns;

mod dns_query;

use dns_query::Message;

use futures::{Future, Poll};
use futures::stream::Stream;

use futures_mio::{Loop, UdpSocket, Sender, Receiver};

use std::thread;
use std::collections::VecDeque;
use std::net::{self, SocketAddr};
use std::io;

type Request = Message;

struct Resolver {
    socket: UdpSocket,
    remote: SocketAddr,
    receiver: Receiver<Request>,
    queue: VecDeque<Request>,
    buffer: Vec<u8>,
}

impl Resolver {
    pub fn new(socket: UdpSocket, remote: SocketAddr, receiver: Receiver<Request>) -> Self {        
        Resolver { socket: socket, remote: remote, receiver: receiver, queue: VecDeque::new(), buffer: Vec::with_capacity(2048) }
    }
}

impl Future for Resolver {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("poll::top");
        println!("  poll self.receiver");
        match self.receiver.poll() {
            Poll::Ok(Some(req)) => self.queue.push_back(req),            
            _ => {},
        }
        println!("  pop queue");
        while let Some(req) = self.queue.pop_front() {
            println!("   poll_write");
            if let Poll::Ok(_) = self.socket.poll_write() {
                println!("   send_message: {:?}", req);
                self.buffer.clear();
                dns_query::encode_message(&mut self.buffer, &req);
                let n = try_nb!(self.socket.send_to(&self.buffer, &self.remote));
                println!("   {} bytes sent", n);
            } else {
                println!("   unsend_message: {:?}", req);
                self.queue.push_front(req);
            }
        }

        println!("  wait for data");
        while let Poll::Ok(_) = self.socket.poll_read() {
            let mut buf = [0u8; 512];
            println!("  reading data");

            let _  = try_nb!(self.socket.recv_from(&mut buf));
            let msg = dns_query::decode_message(&mut buf);
            println!("  message: {:?}", msg);            
        }
        println!("poll::bottom\n");
        Poll::NotReady        
    }
}

fn resolver(remote: SocketAddr) -> (Sender<Request>, thread::JoinHandle<()>)  {    
    use std::sync::mpsc;

    let (txa, rxa) = mpsc::channel::<Sender<Request>>();
    let t = thread::spawn(move || {
            let mut lp = Loop::new().unwrap();            
            let (tx, rx) = lp.handle().channel::<Request>();        
            let rx = lp.run(rx).unwrap();
            let socket_bind = lp.handle().udp_bind(&"0.0.0.0:0".parse().unwrap());
            let socket = lp.run(socket_bind).unwrap();
            
            let resolver = Resolver::new(socket, remote, rx);
            txa.send(tx).unwrap();
            lp.run(resolver).unwrap();
    });     
    (rxa.recv().unwrap(), t)
}


fn main() {
    let (r, t) = resolver("8.8.8.8:53".parse().unwrap());    
    r.send(dns_query::build_query_message(dns_query::any_query("google.com"))).unwrap();
    r.send(dns_query::build_query_message(dns_query::any_query("apple.com"))).unwrap();
    t.join().unwrap();
    //let lp = Loop::new().unwrap();

    //    let v = lp.run(done).unwrap();
    //println!("v: {:?}", v);
}