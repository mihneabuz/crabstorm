use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crabstorm::*;

use super::op::Op;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum KvPayload {
    Txn { txn: Vec<Op> },
    TxnOk { txn: Vec<Op> },
}

pub struct KvNode {
    lists: HashMap<usize, Vec<usize>>,
}

impl KvNode {
    pub fn new() -> Self {
        Self {
            lists: HashMap::new(),
        }
    }
}

impl Node for KvNode {
    type Payload = KvPayload;
    type Event = ();

    fn init(&mut self, _: Init) {}

    fn message(&mut self, message: Message<KvPayload>, sender: Sender<KvPayload>) {
        let dest = message.src;
        let reply = message.body.id;

        match message.body.payload {
            KvPayload::Txn { mut txn } => {
                for op in txn.iter_mut() {
                    match op {
                        Op::Read { key, read } => {
                            *read = self.lists.get(key).cloned();
                        }

                        Op::Append { key, value } => {
                            self.lists.entry(*key).or_default().push(*value);
                        }
                    }
                }

                sender.send(dest, reply, KvPayload::TxnOk { txn });
            }

            _ => unreachable!(),
        }
    }

    fn event(&mut self, _: (), _: Sender<KvPayload>) {}
}
