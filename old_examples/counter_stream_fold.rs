extern crate futures;

use futures::*;
use futures::stream::Stream;

struct CounterStream {
    index: i32,
    limit: i32,
    ready: bool,
}

fn counter(limit: i32) -> CounterStream {
    CounterStream{index: 0, limit: limit, ready: false}
}

impl Stream for CounterStream {
    type Item = i32;
    type Error = ();

    fn poll(&mut self, _task: &mut Task) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.ready {
            return Poll::NotReady;
        }
        self.ready = false;
        if self.index == self.limit {
            return Poll::Ok(None);
        }
        let v = self.index;
        self.index += 1;
        Poll::Ok(Some(v))
    }

    fn schedule(&mut self, task: &mut Task) {
        self.ready = true;
        task.notify();
        return
    }
}


fn main() {
    counter(10)
        .map(|v| v * 2)
        .for_each(|v| {
            println!("v: {:?}", v);
            Ok(())
        })
        .forget();

    counter(10)
        .fold(0, |a, b| finished::<i32,()>(a + b))
        .map(|v| println!("sum: {}", v))
        .forget();
}
