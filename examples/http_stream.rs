extern crate futures;
extern crate futures_io;
extern crate futures_mio;

use futures::Future;
use futures::stream::Stream;
use futures_mio::Loop;



fn main() {
    let mut lp = Loop::new().unwrap();
    let address = "127.0.0.1:5000".parse().unwrap();
    let listener = lp.handle().tcp_listen(&address);

    let server = listener.and_then(|listener| {
        let addr = listener.local_addr().unwrap();
        println!("listening on {}", addr);

        let clients = listener.incoming();

        let welcomes = clients.and_then(|(socket, _peer_addr)| {
            futures_io::write_all(socket, "HTTP/1.1 200 OK\r\n\r\nHello, World")
        });
        welcomes.for_each(|(_socket, _welcome)| {
            Ok(())
        })
    });
    lp.run(server).unwrap();
}
