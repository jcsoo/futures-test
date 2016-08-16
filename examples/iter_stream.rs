extern crate futures;

use futures::*;
use futures::stream::{iter, Stream};



fn main() {
    let r = (0i32..5).map(|v: i32| Ok::<i32,()>(v));
    let rs = iter(r);
    rs.for_each(|v| {
        println!("value: {}", v);
        Ok(())
    }).forget();

}
