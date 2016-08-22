extern crate futures;

use futures::*;

fn forty_two() -> BoxFuture<i32,()> {
    done(Ok(42)).boxed()
}

fn times_two(v: i32) -> BoxFuture<i32, ()> {
    finished(v * 2).boxed()
}

fn main() {
    forty_two()
        .and_then(|v| times_two(v))
        .map(|v| println!("v: {}", v))
        .forget();
}
