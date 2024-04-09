mod node;

pub use node::*;

use std::{
    fs::OpenOptions,
    io::Stdin,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Error, Result};
use futures::{
    io::Lines,
    select,
    stream::{select_all, StreamExt},
    AsyncBufReadExt, FutureExt,
};
use inel::{io::RingBufReader, time::Interval, AsyncRingWriteExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, level_filters::LevelFilter};

pub struct InputHandler {
    input: Lines<RingBufReader<Stdin>>,
}

impl InputHandler {
    fn new() -> Self {
        Self {
            input: RingBufReader::new(std::io::stdin()).lines(),
        }
    }

    async fn next<P>(&mut self) -> Option<Message<P>>
    where
        for<'a> P: Deserialize<'a>,
    {
        self.input.next().await.map(|line| {
            serde_json::from_str(&line.expect("Failed to read line"))
                .expect("Failed to parse message")
        })
    }
}

struct OutputHandler<P> {
    id: usize,
    node: String,
    sender: flume::Sender<(String, Option<usize>, P)>,
    receiver: flume::Receiver<(String, Option<usize>, P)>,
}

impl<P> OutputHandler<P>
where
    P: Serialize,
{
    fn new(node: String) -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            id: 0,
            node,
            sender,
            receiver,
        }
    }

    fn sender(&self) -> Sender<P> {
        Sender::new(self.sender.clone())
    }

    async fn write<M>(&mut self, (dst, reply, payload): (String, Option<usize>, M))
    where
        M: Serialize,
    {
        let message = Message {
            src: self.node.clone(),
            dst,
            body: Body {
                id: Some(self.id),
                reply,
                payload,
            },
        };

        self.id += 1;

        let mut bytes = serde_json::to_vec(&message).expect("Failed to serialize message");
        bytes.push(b'\n');

        std::io::stdout()
            .ring_write_all(bytes)
            .await
            .1
            .expect("Failed to write message");
    }

    async fn flush(&mut self)
    where
        P: Serialize,
    {
        while let Ok(parts) = self.receiver.try_recv() {
            self.write(parts).await;
        }
    }
}

pub struct Runtime<E> {
    intervals: Vec<(Duration, E)>,
    trace_file: Option<PathBuf>,
    trace_level: Option<LevelFilter>,
}

impl<E> Runtime<E> {
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
            trace_file: None,
            trace_level: None,
        }
    }

    pub fn event(mut self, time: Duration, event: E) -> Self {
        self.intervals.push((time, event));
        self
    }

    pub fn trace_file(mut self, file: impl AsRef<Path>) -> Self {
        self.trace_file = Some(file.as_ref().to_owned());
        self
    }

    pub fn trace_level(mut self, level: LevelFilter) -> Self {
        self.trace_level = Some(level);
        self
    }

    pub fn run<N, P>(self, mut node: N) -> Result<()>
    where
        E: Clone + 'static,
        N: Node<Payload = P, Event = E> + 'static,
        for<'a> P: Deserialize<'a> + Serialize,
    {
        if let Some(file) = self.trace_file {
            let writer = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(file)
                .expect("Failed to create trace file");

            let level = self.trace_level.unwrap_or(LevelFilter::INFO);

            tracing_subscriber::fmt()
                .with_max_level(level)
                .with_writer(writer)
                .compact()
                .init();
        }

        debug!("Starting node");

        inel::block_on(async move {
            let mut input = InputHandler::new();
            let mut events = select_all(
                self.intervals
                    .into_iter()
                    .map(|i| Interval::new(i.0).map(move |_| i.1.clone())),
            );

            let init_message = input.next::<InitPayload>().await.unwrap();

            let InitPayload::Init(init) = init_message.body.payload else {
                return Err(Error::msg("bad init message"));
            };

            let mut output = OutputHandler::new(init.node_id.clone());

            node.init(init);

            output
                .write((init_message.src, init_message.body.id, InitPayload::InitOk))
                .await;

            loop {
                output.flush().await;

                select! {
                    message = input.next().fuse() => {
                        if let Some(message) = message {
                            debug!("Message");
                            node.message(message, output.sender());
                        } else {
                            break;
                        }
                    }

                    event = events.next() => {
                        if let Some(event) = event {
                            debug!("Event");
                            node.event(event, output.sender());
                        }
                    },
                };
            }

            Ok(())
        })
    }
}

impl<E> Default for Runtime<E> {
    fn default() -> Self {
        Self::new()
    }
}
