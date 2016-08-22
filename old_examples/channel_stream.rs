extern crate futures;

use futures::*;
use futures::stream::{channel, Stream};



fn main() {
    let (tx, rx) = channel::<i32, ()>();
    tx
        .send(Ok(0))
        .and_then(|t| t.send(Ok(1)))
        .and_then(|t| t.send(Ok(2)))
        .and_then(|t| t.send(Ok(3)))
        .forget();

    rx
        .for_each(|v| {
            println!("v: {}", v);
            Ok(())
        })
        .forget();

}
