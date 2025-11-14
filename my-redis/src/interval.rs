use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::Stream;
use crate::delay::Delay;

pub struct Interval {
    pub rem: usize,
    pub delay: Delay,
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.rem == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.delay).poll(cx) {
            Poll::Ready(_) => {
                let when = self.delay.when + Duration::from_secs(1);
                self.delay = Delay { when };
                self.rem -= 1;
                Poll::Ready(Some(()))
            } 
            Poll::Pending => Poll::Pending,
        }
    }
}