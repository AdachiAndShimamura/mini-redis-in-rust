use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio_stream::{ StreamMap};
use crate::base::event::Message;

pub struct Subscribe {
    pub(crate) channels: Vec<String>,
}

impl Subscribe {
    pub fn from_vec(channels:Vec<String>)->Self{
        Subscribe{
            channels
        }
    }
}

pub struct  Subscriber{
    pub subscribers:StreamMap<String,Message>
}


impl Subscriber {

}


impl Stream for Subscriber {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}