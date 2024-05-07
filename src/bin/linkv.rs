use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum LinkvPayload {
    Read { key: Value },
    ReadOk { value: Option<Value> },
    Write { key: Value, value: Value },
    WriteOk,
    Cas { key: Value, from: Value, to: Value },
    CasOk,
    Error { code: usize },
    Raft { rpc: raft::Rpc },
}

#[derive(Clone, Debug)]
enum LinkvEvent {
    RaftTick,
}

const KEY_MISSING: usize = 20;
const VALUE_MISMATCH: usize = 22;

macro_rules! error {
    ($code:expr) => {
        LinkvPayload::Error { code: $code }
    };
}

struct LinkvNode {
    store: HashMap<Value, Value>,
    raft: Option<raft::Raft>,
}

impl LinkvNode {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
            raft: None,
        }
    }

    fn send_raft(&self, delivery: raft::Delivery, sender: Sender<LinkvPayload>) {
        match delivery {
            raft::Delivery::Broadcast(rpc) => {
                let raft = self.raft.as_ref().unwrap();
                let payload = LinkvPayload::Raft { rpc };
                for node in raft.nodes().iter().filter(|node| *node != raft.id()) {
                    sender.send(node.clone(), None, payload.clone());
                }
            }

            raft::Delivery::Unicast(dest, rpc) => {
                sender.send(dest, None, LinkvPayload::Raft { rpc });
            }
        }
    }
}

impl Node for LinkvNode {
    type Payload = LinkvPayload;
    type Event = LinkvEvent;

    fn init(&mut self, init: Init) {
        self.raft = Some(raft::Raft::new(init.id, init.nodes));
    }

    fn message(&mut self, message: Message<LinkvPayload>, sender: Sender<LinkvPayload>) {
        let dest = message.src;
        let reply = message.body.id;

        match message.body.payload {
            LinkvPayload::Read { key } => {
                let value = self.store.get(&key).cloned();

                sender.send(dest, reply, LinkvPayload::ReadOk { value });
            }

            LinkvPayload::Write { key, value } => {
                self.store.insert(key, value);

                sender.send(dest, reply, LinkvPayload::WriteOk);
            }

            LinkvPayload::Cas { key, from, to } => {
                let mut res = LinkvPayload::CasOk;

                if let Some(value) = self.store.get_mut(&key) {
                    if *value == from {
                        *value = to;
                    } else {
                        res = error!(VALUE_MISMATCH);
                    }
                } else {
                    res = error!(KEY_MISSING);
                }

                sender.send(dest, reply, res);
            }

            LinkvPayload::Raft { rpc } => {
                let raft = self.raft.as_mut().unwrap();

                if let Some(delivery) = raft.process(dest, rpc) {
                    self.send_raft(delivery, sender);
                }
            }

            _ => unreachable!(),
        }
    }

    fn event(&mut self, event: Self::Event, sender: Sender<LinkvPayload>) {
        match event {
            LinkvEvent::RaftTick => {
                if let Some(delivery) = self.raft.as_mut().unwrap().tick() {
                    self.send_raft(delivery, sender);
                }
            }
        };
    }
}

fn main() {
    Runtime::new()
        .event(Duration::from_millis(200), LinkvEvent::RaftTick)
        .run(LinkvNode::new())
        .unwrap()
}
