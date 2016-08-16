extern crate futures;

use futures::*;

struct MyFuture {
    value: i32
}

fn foo(v: i32) -> MyFuture {
    MyFuture {value: v}
}

impl Future for MyFuture {
    type Item = i32;
    type Error = ();

    fn poll(&mut self, _task: &mut Task) -> Poll<Self::Item, Self::Error> {
        Poll::Ok(self.value)
    }

    fn schedule(&mut self, _task: &mut Task) {
        return
    }
}


fn main() {
    foo(42).map(|v| println!("v: {}", v)).forget();
}
