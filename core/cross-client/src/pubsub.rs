use std::collections::{HashMap, VecDeque};
use std::task::{Context, Poll};
use std::{io, marker::PhantomData, pin::Pin};

use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use serde::{de, Deserialize, Serialize};
use tokio_util::codec::Framed;

use protocol::tokio::io::{AsyncRead, AsyncWrite};
use protocol::types::BytesMut;

use crate::stream_codec::StreamCodec;

/// General rpc subscription client
pub struct Client<T> {
    inner: Framed<T, StreamCodec>,
    id:    usize,
}

impl<T> Client<T>
where
    T: AsyncWrite + AsyncRead + Unpin + Sync + Send,
{
    /// New a pubsub rpc client
    pub fn new(io: T) -> Client<T> {
        let inner = Framed::new(io, StreamCodec::stream_incoming());
        Client { inner, id: 0 }
    }

    /// Subscription a topic
    pub async fn subscribe<F: for<'de> de::Deserialize<'de>>(
        mut self,
        name: &str,
    ) -> io::Result<Handle<T, F>> {
        let mut topic_list = HashMap::default();
        let mut pending_recv = VecDeque::new();

        subscribe(
            &mut self.inner,
            self.id,
            name,
            &mut topic_list,
            &mut pending_recv,
        )
        .await?;
        self.id = self.id.wrapping_add(1);

        Ok(Handle {
            inner: self.inner,
            topic_list,
            output: PhantomData::default(),
            rpc_id: self.id,
            pending_recv,
        })
    }

    /// Subscription topics
    pub async fn subscribe_list<
        F: for<'de> de::Deserialize<'de>,
        I: Iterator<Item = H>,
        H: AsRef<str>,
    >(
        mut self,
        name_list: I,
    ) -> io::Result<Handle<T, F>> {
        let mut topic_list = HashMap::default();
        let mut pending_recv = VecDeque::new();

        for topic in name_list {
            subscribe(
                &mut self.inner,
                self.id,
                topic,
                &mut topic_list,
                &mut pending_recv,
            )
            .await?;
            self.id = self.id.wrapping_add(1);
        }

        Ok(Handle {
            inner: self.inner,
            topic_list,
            output: PhantomData::default(),
            rpc_id: self.id,
            pending_recv,
        })
    }
}

/// General rpc subscription topic handle
pub struct Handle<T, F> {
    inner:        Framed<T, StreamCodec>,
    topic_list:   HashMap<String, String>,
    output:       PhantomData<F>,
    rpc_id:       usize,
    pending_recv: VecDeque<BytesMut>,
}

impl<T, F> Handle<T, F>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    /// Sub ids
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.topic_list.keys()
    }

    /// Topic names
    pub fn topics(&self) -> impl Iterator<Item = &String> {
        self.topic_list.values()
    }

    /// if topic is empty, return Ok, else Err
    pub fn try_into(self) -> Result<Client<T>, Self> {
        if self.topic_list.is_empty() {
            Ok(Client {
                inner: self.inner,
                id:    self.rpc_id,
            })
        } else {
            Err(self)
        }
    }

    pub async fn subscribe(mut self, topic: &str) -> io::Result<Self> {
        if self.topic_list.iter().any(|(_, v)| *v == topic) {
            return Ok(self);
        }

        subscribe(
            &mut self.inner,
            self.rpc_id,
            topic,
            &mut self.topic_list,
            &mut self.pending_recv,
        )
        .await?;
        self.rpc_id = self.rpc_id.wrapping_add(1);

        Ok(self)
    }

    /// Unsubscribe one topic
    pub async fn unsubscribe(&mut self, topic: &str) -> io::Result<()> {
        let id = {
            let id = self
                .topic_list
                .iter()
                .find_map(|(k, v)| if v == topic { Some(k) } else { None })
                .cloned();
            if id.is_none() {
                return Ok(());
            }
            id.unwrap()
        };
        let req_json = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "unsubscribe", "params": ["{}"]}}"#,
            self.rpc_id, id
        );
        self.rpc_id = self.rpc_id.wrapping_add(1);

        self.inner.send(req_json).await?;

        let output = loop {
            let resp = self.inner.next().await;

            let resp = resp.ok_or_else::<io::Error, _>(|| io::ErrorKind::BrokenPipe.into())??;

            // Since the current subscription state, the return value may be a notification,
            // we need to ensure that the unsubscribed message returns before jumping out
            match serde_json::from_slice::<jsonrpc_core::response::Output>(&resp) {
                Ok(output) => break output,
                Err(_) => self.pending_recv.push_back(resp),
            }
        };

        match output {
            jsonrpc_core::response::Output::Success(_) => {
                self.topic_list.remove(&id);
                Ok(())
            }
            jsonrpc_core::response::Output::Failure(e) => {
                Err(io::Error::new(io::ErrorKind::InvalidData, e.error))
            }
        }
    }

    /// Unsubscribe and return this Client
    pub async fn unsubscribe_all(mut self) -> io::Result<Client<T>> {
        for topic in self.topic_list.clone().values() {
            self.unsubscribe(topic).await?
        }
        Ok(Client {
            inner: self.inner,
            id:    self.rpc_id,
        })
    }
}

impl<T, F> Stream for Handle<T, F>
where
    F: for<'de> serde::de::Deserialize<'de> + Unpin,
    T: AsyncWrite + AsyncRead + Unpin,
{
    type Item = io::Result<(String, F)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let parse =
            |data: BytesMut, topic_list: &HashMap<String, String>| -> io::Result<(String, F)> {
                let output = serde_json::from_slice::<jsonrpc_core::request::Notification>(&data)
                    .expect("must parse to notification");
                let message = output
                    .params
                    .parse::<Message>()
                    .expect("must parse to message");
                serde_json::from_str::<F>(&message.result)
                    .map(|r| (topic_list.get(&message.subscription).cloned().unwrap(), r))
                    .map_err(|_| io::ErrorKind::InvalidData.into())
            };

        if let Some(data) = self.pending_recv.pop_front() {
            return Poll::Ready(Some(parse(data, &self.topic_list)));
        }
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(parse(frame, &self.topic_list))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Message {
    result:       String,
    subscription: String,
}

async fn subscribe<T: AsyncWrite + AsyncRead + Unpin>(
    io: &mut Framed<T, StreamCodec>,
    id: usize,
    topic: impl AsRef<str>,
    topic_list: &mut HashMap<String, String>,
    pending_recv: &mut VecDeque<BytesMut>,
) -> io::Result<()> {
    // telnet localhost 18114
    // > {"id": 2, "jsonrpc": "2.0", "method": "subscribe", "params":
    // ["new_tip_header"]} < {"jsonrpc":"2.0","result":0,"id":2}
    // < {"jsonrpc":"2.0","method":"subscribe","params":{"result":"...block header
    // json...", "subscription":0}}
    // < {"jsonrpc":"2.0","method":"subscribe","params":{"result":"...block header
    // json...", "subscription":0}}
    // < ...
    // > {"id": 2, "jsonrpc": "2.0", "method": "unsubscribe", "params": [0]}
    // < {"jsonrpc":"2.0","result":true,"id":2}
    let req_json = format!(
        r#"{{"id": {}, "jsonrpc": "2.0", "method": "subscribe", "params": ["{}"]}}"#,
        id,
        topic.as_ref()
    );

    io.send(req_json).await?;

    // loop util this subscribe success
    loop {
        let resp = io.next().await;
        let resp = resp.ok_or_else::<io::Error, _>(|| io::ErrorKind::BrokenPipe.into())??;
        match serde_json::from_slice::<jsonrpc_core::response::Output>(&resp) {
            Ok(output) => match output {
                jsonrpc_core::response::Output::Success(success) => {
                    let res = serde_json::from_value::<String>(success.result).unwrap();
                    topic_list.insert(res, topic.as_ref().to_owned());
                    break Ok(());
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e.error))
                }
            },
            // must be Notification message
            Err(_) => pending_recv.push_back(resp),
        }
    }
}
