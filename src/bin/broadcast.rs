use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::Result;
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
    Gossip {
        messages: Vec<usize>,
    },
}

struct BroadcastNode {
    id: String,
    neigs: Vec<String>,
    set: HashSet<usize>,

    seen: HashMap<String, HashSet<usize>>,
}

impl BroadcastNode {
    fn new() -> Self {
        Self {
            id: "".to_string(),
            neigs: Vec::new(),
            set: HashSet::new(),
            seen: HashMap::new(),
        }
    }
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn oninit(&mut self, init: Init) -> Result<()> {
        self.id = init.node_id;
        self.seen = HashMap::from_iter(
            init.node_ids
                .iter()
                .map(|node| (node.clone(), HashSet::new())),
        );
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
            }

            BroadcastPayload::Gossip { messages } => {
                let seen = self.seen.get_mut(&dst).unwrap();
                seen.extend(messages.iter().copied());
                self.set.extend(messages.into_iter());
            }

            _ => unreachable!(),
        };

        Ok(())
    }

    fn onevent(&mut self, _: (), sender: &mut Sender) -> Result<()> {
        for neigh in self.neigs.iter() {
            let seen = self.seen.get(neigh).unwrap();
            let to_send = self.set.difference(seen).copied().collect::<Vec<_>>();

            if !to_send.is_empty() {
                sender.send(
                    neigh.clone(),
                    None,
                    BroadcastPayload::Gossip { messages: to_send },
                )?;
            }
        }

        Ok(())
    }
}

fn main() {
    Runtime::new(BroadcastNode::new())
        .event(Duration::from_millis(800), ())
        .run()
        .unwrap()
}
