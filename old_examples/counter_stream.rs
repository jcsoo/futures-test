extern crate futures;

use futures::*;
use futures::stream::Stream;

struct CounterStream {
    index: i32,
    limit: i32,
}

fn counter(limit: i32) -> CounterStream {
    CounterStream{index: 0, limit: limit}
}

impl Stream for CounterStream {
    type Item = i32;
    type Error = ();

    fn poll(&mut self, _task: &mut Task) -> Poll<Option<Self::Item>, Self::Error> {
        if self.index == self.limit {
            return Poll::Ok(None);
        }
        let v = self.index;
        self.index += 1;
        Poll::Ok(Some(v))
    }

    fn schedule(&mut self, _task: &mut Task) {
        return
    }
}


fn main() {
    counter(10).for_each(|v| {
        println!("v: {:?}", v);
        Ok(())
    }).forget();
}
