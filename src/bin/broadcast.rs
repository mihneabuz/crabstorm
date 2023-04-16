use std::collections::{HashMap, HashSet};

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    id: String,
    neigs: Vec<String>,
    set: HashSet<usize>,
}

impl BroadcastNode {
    fn new() -> Self {
        Self {
            id: "".to_string(),
            neigs: Vec::new(),
            set: HashSet::new(),
        }
    }
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn oninit(&mut self, init: Init) -> Result<()> {
        self.id = init.node_id;
        self.neigs = init.node_ids;
        Ok(())
    }

    fn onmessage(&mut self, message: Message<BroadcastPayload>, sender: &mut Sender) -> Result<()> {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            BroadcastPayload::Broadcast { message } => {
                self.set.insert(message);
                sender.send(dst, rply, BroadcastPayload::BroadcastOk)?;
            }

            BroadcastPayload::Read => {
                let messages = self.set.iter().copied().collect();
                sender.send(dst, rply, BroadcastPayload::ReadOk { messages })?;
            }

            BroadcastPayload::Topology { mut topology } => {
                self.neigs = topology.remove(&self.id).unwrap();
                sender.send(dst, rply, BroadcastPayload::TopologyOk)?;
            },

            _ => { unreachable!() }
        };

        Ok(())
    }
}

fn main() {
    Runtime::new().run(BroadcastNode::new()).unwrap()
}
