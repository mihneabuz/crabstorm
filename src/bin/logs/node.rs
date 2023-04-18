use anyhow::Result;
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

impl Node<LogPayload> for LogNode {
    fn oninit(&mut self, init: Init) -> Result<()> {
        self.id = init.node_id;
        Ok(())
    }

    fn onmessage(&mut self, message: Message<LogPayload>, sender: &mut Sender) -> Result<()> {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            LogPayload::Send { key, msg } => {
                let log = self.logs.entry(key).or_default();
                let offset = log.push(msg);

                sender.send(dst, rply, LogPayload::SendOk { offset })?;
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

                sender.send(dst, rply, LogPayload::PollOk { msgs })?;
            }

            LogPayload::CommitOffsets { offsets } => {
                for (name, offset) in offsets {
                    self.logs.get_mut(&name).map(|log| log.commit(offset));
                }

                sender.send(dst, rply, LogPayload::CommitOffsetsOk)?;
            }

            LogPayload::ListCommittedOffsets { keys } => {
                let offsets = keys
                    .into_iter()
                    .filter_map(|name| self.logs.get(&name).map(|log| (name, log.commited())))
                    .collect();

                sender.send(dst, rply, LogPayload::ListCommittedOffsetsOk { offsets })?;
            }

            _ => unreachable!(),
        }

        Ok(())
    }

    fn onevent(&mut self, _: (), _: &mut Sender) -> Result<()> {
        Ok(())
    }
}
