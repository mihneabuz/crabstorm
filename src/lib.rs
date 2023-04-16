use anyhow::{Error, Result};
use futures::io::{AsyncBufReadExt, BufReader};
use futures::stream::{select_all, StreamExt};
use futures::{select, FutureExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smol::{Timer, Unblock};
use std::io::{self, Write};
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub rply: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<Payload: DeserializeOwned + Serialize, Event = ()> {
    fn oninit(&mut self, init: Init) -> Result<()>;
    fn onmessage(&mut self, message: Message<Payload>, sender: &mut Sender) -> Result<()>;
    fn onevent(&mut self, event: Event, sender: &mut Sender) -> Result<()>;
}

pub struct Sender {
    id: usize,
    node: String,
}

impl Sender {
    pub fn new() -> Self {
        Self {
            id: 0,
            node: "".to_string(),
        }
    }

    pub fn send(&mut self, dst: String, rply: Option<usize>, pl: impl Serialize) -> Result<()> {
        let mut stdout = io::stdout().lock();

        let message = Message {
            src: self.node.clone(),
            dst,
            body: Body {
                id: Some(self.id),
                rply,
                payload: pl,
            },
        };

        self.id += 1;

        stdout.write_all(&serde_json::to_vec(&message)?)?;
        stdout.write_all(b"\n")?;

        Ok(())
    }
}

#[derive(Clone)]
struct Interval<E> {
    time: Duration,
    event: E,
}

pub struct Runtime<P, E, N> {
    node: N,
    sender: Sender,
    intervals: Vec<Interval<E>>,
    _p: PhantomData<P>,
}

impl<P, E, N> Runtime<P, E, N>
where
    E: Copy,
    P: DeserializeOwned + Serialize,
    N: Node<P, E>,
{
    pub fn new(node: N) -> Self {
        Self {
            node,
            sender: Sender::new(),
            intervals: Vec::new(),
            _p: PhantomData,
        }
    }

    pub fn event(mut self, time: Duration, event: E) -> Self {
        self.intervals.push(Interval { time, event });
        self
    }

    pub fn run(&mut self) -> Result<()> {
        let mut input = BufReader::new(Unblock::new(io::stdin())).lines();

        smol::block_on(async {
            let first = input.next().await.unwrap()?;
            let init_message: Message<InitPayload> = serde_json::from_str(&first)?;

            let InitPayload::Init(init) = init_message.body.payload else {
                return Err(Error::msg("bad init message"));
            };

            self.sender.node = init.node_id.clone();

            self.node.oninit(init)?;
            self.sender
                .send(init_message.src, init_message.body.id, InitPayload::InitOk)?;

            let mut intervals = select_all(
                self.intervals
                    .iter()
                    .map(|i| StreamExt::map(Timer::interval(i.time), |_| i.event)),
            );

            loop {
                select! {
                    int = intervals.next() => {
                        if let Some(event) = int {
                            self.node.onevent(event, &mut self.sender)?;
                        }
                    },

                    res = input.next().fuse() => {
                        if res.is_none() {
                            break;
                        }

                        let message = serde_json::from_str(&res.unwrap()?)?;
                        self.node.onmessage(message, &mut self.sender)?;
                    }
                }
            }

            Ok(())
        })
    }
}
