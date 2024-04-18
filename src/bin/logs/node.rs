use serde::{Deserialize, Serialize};
use std::{collections::HashMap, unreachable};

use crate::log::Log;
use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum LogPayload {
    Send {
        key: String,
        msg: i32,
    },
    SendOk {
        offset: i32,
    },
    Poll {
        offsets: HashMap<String, i32>,
    },
    PollOk {
        msgs: HashMap<String, Vec<[i32; 2]>>,
    },
    CommitOffsets {
        offsets: HashMap<String, i32>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, i32>,
    },
}

pub struct LogNode {
    id: String,
    logs: HashMap<String, Log>,
}

impl LogNode {
    pub fn new() -> Self {
        Self {
            id: String::default(),
            logs: HashMap::new(),
        }
    }
}

impl Node for LogNode {
    type Payload = LogPayload;
    type Event = ();

    fn init(&mut self, init: Init) {
        self.id = init.id;
    }

    fn message(&mut self, message: Message<LogPayload>, sender: Sender<LogPayload>) {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            LogPayload::Send { key, msg } => {
                let log = self.logs.entry(key).or_default();
                let offset = log.push(msg);

                sender.send(dst, rply, LogPayload::SendOk { offset });
            }

            LogPayload::Poll { offsets } => {
                let msgs = offsets
                    .into_iter()
                    .filter_map(|(name, offset)| {
                        self.logs.get(&name).map(|log| {
                            (
                                name,
                                log.poll(offset).into_iter().map(|(a, b)| [a, b]).collect(),
                            )
                        })
                    })
                    .collect();

                sender.send(dst, rply, LogPayload::PollOk { msgs });
            }

            LogPayload::CommitOffsets { offsets } => {
                for (name, offset) in offsets {
                    if let Some(log) = self.logs.get_mut(&name) {
                        log.commit(offset);
                    }
                }

                sender.send(dst, rply, LogPayload::CommitOffsetsOk);
            }

            LogPayload::ListCommittedOffsets { keys } => {
                let offsets = keys
                    .into_iter()
                    .filter_map(|name| self.logs.get(&name).map(|log| (name, log.commited())))
                    .collect();

                sender.send(dst, rply, LogPayload::ListCommittedOffsetsOk { offsets });
            }

            _ => unreachable!(),
        }
    }

    fn event(&mut self, _: (), _: Sender<LogPayload>) {}
}
