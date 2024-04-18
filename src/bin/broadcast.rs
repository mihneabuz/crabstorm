use std::collections::{HashMap, HashSet};
use std::time::Duration;

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

impl Node for BroadcastNode {
    type Payload = BroadcastPayload;
    type Event = ();

    fn init(&mut self, init: Init) {
        self.id = init.id;
        self.seen = HashMap::from_iter(
            init.neighbors
                .iter()
                .map(|node| (node.clone(), HashSet::new())),
        );
        self.neigs = init.neighbors;
    }

    fn message(&mut self, message: Message<BroadcastPayload>, sender: Sender<BroadcastPayload>) {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            BroadcastPayload::Broadcast { message } => {
                self.set.insert(message);
                sender.send(dst, rply, BroadcastPayload::BroadcastOk);
            }

            BroadcastPayload::Read => {
                let messages = self.set.iter().copied().collect();
                sender.send(dst, rply, BroadcastPayload::ReadOk { messages });
            }

            BroadcastPayload::Topology { mut topology } => {
                self.neigs = topology.remove(&self.id).unwrap();
                sender.send(dst, rply, BroadcastPayload::TopologyOk);
            }

            BroadcastPayload::Gossip { messages } => {
                let seen = self.seen.get_mut(&dst).unwrap();
                seen.extend(messages.iter().copied());
                self.set.extend(messages);
            }

            _ => unreachable!(),
        };
    }

    fn event(&mut self, _: (), sender: Sender<BroadcastPayload>) {
        for neigh in self.neigs.iter() {
            let seen = self.seen.get(neigh).unwrap();
            let to_send = self.set.difference(seen).copied().collect::<Vec<_>>();

            if !to_send.is_empty() {
                sender.send(
                    neigh.clone(),
                    None,
                    BroadcastPayload::Gossip { messages: to_send },
                );
            }
        }
    }
}

fn main() {
    Runtime::new()
        .event(Duration::from_millis(200), ())
        .run(BroadcastNode::new())
        .unwrap()
}
