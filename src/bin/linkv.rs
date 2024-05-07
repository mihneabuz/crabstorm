use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
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
}

const KEY_MISSING: usize = 20;
const VALUE_MISMATCH: usize = 22;

struct LinkvNode {
    store: HashMap<Value, Value>,
}

impl LinkvNode {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
}

impl Node for LinkvNode {
    type Payload = LinkvPayload;
    type Event = ();

    fn init(&mut self, _: Init) {}

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
                        res = LinkvPayload::Error {
                            code: VALUE_MISMATCH,
                        };
                    }
                } else {
                    res = LinkvPayload::Error { code: KEY_MISSING };
                }

                sender.send(dest, reply, res);
            }

            _ => unreachable!(),
        }
    }
}

fn main() {
    Runtime::new().run(LinkvNode::new()).unwrap()
}
