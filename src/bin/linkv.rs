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
    Raft { rpc: raft::Rpc<RaftCommand> },
}

#[derive(Clone, Debug)]
enum LinkvEvent {
    RaftTick,
    Debug,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Command {
    Write { key: Value, value: Value },
    Cas { key: Value, from: Value, to: Value },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RaftCommand {
    origin: String,
    reply: (usize, String),
    command: Command,
}

const KEY_MISSING: usize = 20;
const VALUE_MISMATCH: usize = 22;

macro_rules! error {
    ($e:expr) => {
        LinkvPayload::Error { code: $e }
    };
}

struct LinkvNode {
    store: HashMap<Value, Value>,
    raft: raft::Raft<RaftCommand>,
}

impl LinkvNode {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
            raft: raft::Raft::new("nop".to_string(), vec![]),
        }
    }

    fn apply_raft(&mut self, command: RaftCommand, sender: &Sender<LinkvPayload>) {
        if let Some(delivery) = self.raft.apply(command) {
            self.send_raft(delivery, sender);
        }
    }

    fn send_raft(&self, delivery: raft::Delivery<RaftCommand>, sender: &Sender<LinkvPayload>) {
        match delivery {
            raft::Delivery::Unicast(dest, rpc) => {
                sender.send(dest, None, LinkvPayload::Raft { rpc });
            }

            raft::Delivery::Broadcast(rpc) => {
                let payload = LinkvPayload::Raft { rpc };
                for node in self.raft.others() {
                    sender.send(node.clone(), None, payload.clone());
                }
            }

            raft::Delivery::Multicast(rpcs) => {
                for (dest, rpc) in rpcs.into_iter() {
                    sender.send(dest, None, LinkvPayload::Raft { rpc });
                }
            }
        }
    }
}

impl Node for LinkvNode {
    type Payload = LinkvPayload;
    type Event = LinkvEvent;

    fn init(&mut self, init: Init) {
        self.raft = raft::Raft::new(init.id, init.nodes);
    }

    fn message(&mut self, message: Message<LinkvPayload>, sender: Sender<LinkvPayload>) {
        let id = message.body.id;
        let dest = message.src;

        match message.body.payload {
            LinkvPayload::Read { key } => {
                let value = self.store.get(&key).cloned();
                sender.send(dest, id, LinkvPayload::ReadOk { value });
            }

            LinkvPayload::Write { key, value } => {
                let command = RaftCommand {
                    origin: self.raft.id().clone(),
                    reply: (id.unwrap(), dest),
                    command: Command::Write { key, value },
                };

                self.apply_raft(command, &sender);
            }

            LinkvPayload::Cas { key, from, to } => {
                let command = RaftCommand {
                    origin: self.raft.id().clone(),
                    reply: (id.unwrap(), dest),
                    command: Command::Cas { key, from, to },
                };

                self.apply_raft(command, &sender);
            }

            LinkvPayload::Raft { rpc } => {
                if let Some(delivery) = self.raft.process(dest, rpc) {
                    self.send_raft(delivery, &sender);
                }

                while let Some(action) = self.raft.consume() {
                    let (reply, dest) = action.reply;

                    match action.command {
                        Command::Write { key, value } => {
                            self.store.insert(key.clone(), value.clone());

                            if action.origin == *self.raft.id() {
                                sender.send(dest, Some(reply), LinkvPayload::WriteOk);
                            }
                        }

                        Command::Cas { key, from, to } => {
                            let mut result = LinkvPayload::CasOk;

                            if let Some(entry) = self.store.get_mut(&key) {
                                if *entry == from {
                                    *entry = to.clone();
                                } else {
                                    result = error!(VALUE_MISMATCH);
                                }
                            } else {
                                result = error!(KEY_MISSING);
                            }

                            if action.origin == *self.raft.id() {
                                sender.send(dest, Some(reply), result);
                            }
                        }
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    fn event(&mut self, event: Self::Event, sender: Sender<LinkvPayload>) {
        match event {
            LinkvEvent::RaftTick => {
                if let Some(delivery) = self.raft.tick() {
                    self.send_raft(delivery, &sender);
                }
            }

            LinkvEvent::Debug => {
                eprintln!("STATE: {:?}", self.store);
            }
        };
    }
}

fn main() {
    Runtime::new()
        .event(Duration::from_millis(50), LinkvEvent::RaftTick)
        .event(Duration::from_millis(1000), LinkvEvent::Debug)
        .run(LinkvNode::new())
        .unwrap()
}
