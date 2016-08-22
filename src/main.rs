#![allow(unused_imports, unused_variables, dead_code)]

extern crate futures;
#[macro_use]
extern crate futures_io;
extern crate futures_mio;
extern crate trust_dns;

mod dns_query;

use dns_query::Message;

use futures::{Future, Poll, oneshot, Oneshot, Complete};
use futures::stream::Stream;

use futures_mio::{Loop, UdpSocket, Sender, Receiver};

use std::thread;
use std::collections::{VecDeque, HashMap};
use std::net::{self, SocketAddr};
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

static DNS_REQ_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn next_request_id() -> u16 {
    DNS_REQ_ID.fetch_add(1, Ordering::Relaxed) as u16
}

type Request = (Message, Complete<Message>);

struct Resolver {
    socket: UdpSocket,
    remote: SocketAddr,
    receiver: Receiver<Request>,
    queue: VecDeque<Request>,
    requests: HashMap<u16, Complete<Message>>,
    buffer: Vec<u8>,
    
}

struct ResolverHandle {
    sender: Sender<Request>,
    thread: thread::JoinHandle<()>,
}

impl Resolver {
    pub fn new(socket: UdpSocket, remote: SocketAddr, receiver: Receiver<Request>) -> Self {        
        Resolver { 
            socket: socket, 
            remote: remote, 
            receiver: receiver, 
            queue: VecDeque::new(),
            requests: HashMap::new(), 
            buffer: Vec::with_capacity(2048) }
    }
}

impl Future for Resolver {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Poll::Ok(Some(req)) = self.receiver.poll() {
            self.queue.push_back(req)            
        }
        while let Some((mut req, complete)) = self.queue.pop_front() {
            if let Poll::Ok(_) = self.socket.poll_write() {
                self.buffer.clear();
                let id = next_request_id();
                req.id(id);
                dns_query::encode_message(&mut self.buffer, &req);
                let n = try_nb!(self.socket.send_to(&self.buffer, &self.remote));
                self.requests.insert(id, complete);
            } else {
                self.queue.push_front((req, complete));
            }
        }

        while let Poll::Ok(_) = self.socket.poll_read() {
            let mut buf = [0u8; 512];
            let (n, addr)  = try_nb!(self.socket.recv_from(&mut buf));
            if addr != self.remote {
                println!("Discarding message from unexpected address: want {:?}, got {:?}", self.remote, addr);
            }
            let msg = dns_query::decode_message(&mut buf);
            if let Some(complete) = self.requests.remove(&msg.get_id()) {
                complete.complete(msg)
            } else {
            }
                        
        }
        Poll::NotReady        
    }
}

impl ResolverHandle {
    pub fn query(&self, q: Message) -> Oneshot<Message> {
        let (c, p) = oneshot::<Message>();   
        self.sender.send((q, c)).unwrap();
        p
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

fn resolver(remote: SocketAddr) -> ResolverHandle  {    
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
    ResolverHandle {
        sender: rxa.recv().unwrap(),
        thread: t,
    } 
}


fn main() {
    let r = resolver("8.8.8.8:53".parse().unwrap());
        
    let q = dns_query::build_query_message(dns_query::any_query("google.com"));
    let p1 = r.query(q);   
   
    let q = dns_query::build_query_message(dns_query::any_query("apple.com"));
    let p2 = r.query(q);

    p1.map(|m| {
        println!("c1 got message:{:?}\n", m);
    }).forget();

    p2.map(|m| {
        println!("c2 got message:{:?}\n", m);
    }).forget();
    
    r.join();
}