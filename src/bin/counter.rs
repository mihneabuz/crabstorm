use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum CounterPayload {
    Add { delta: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
    Gossip { value: usize },
    GossipOk { value: usize },
}

struct CounterNode {
    id: String,
    acc: usize,

    // this contains all the other nodes
    // for every node we store 2 values
    //  1. the counter we *currently* know the other node has
    //  2. the latest value that the other node has confirmed to have received from us
    others: HashMap<String, (usize, usize)>,
}

impl CounterNode {
    fn new() -> Self {
        Self {
            id: String::default(),
            acc: 0,
            others: HashMap::new(),
        }
    }
}

impl Node for CounterNode {
    type Payload = CounterPayload;
    type Event = ();

    fn init(&mut self, init: Init) {
        self.id = init.node_id;
        self.others.extend(
            init.node_ids
                .into_iter()
                .filter(|n| *n != self.id)
                .map(|n| (n, (0, 0))),
        );
    }

    fn message(&mut self, message: Message<CounterPayload>, sender: Sender<CounterPayload>) {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            CounterPayload::Add { delta } => {
                self.acc += delta;
                sender.send(dst, rply, CounterPayload::AddOk);
            }

            CounterPayload::Read => {
                let value = self.acc + self.others.values().map(|e| e.0).sum::<usize>();
                sender.send(dst, rply, CounterPayload::ReadOk { value });
            }

            CounterPayload::Gossip { value } => {
                self.others
                    .entry(dst.clone())
                    .and_modify(|(acc, _)| *acc = cmp::max(value, *acc));

                sender.send(dst, rply, CounterPayload::GossipOk { value });
            }

            CounterPayload::GossipOk { value } => {
                self.others
                    .entry(dst)
                    .and_modify(|(_, confirmed)| *confirmed = cmp::max(value, *confirmed));
            }

            _ => unimplemented!(),
        };
    }

    fn event(&mut self, _: (), sender: Sender<CounterPayload>) {
        for (n, _) in self.others.iter().filter(|(_, &(_, conf))| conf < self.acc) {
            sender.send(n.clone(), None, CounterPayload::Gossip { value: self.acc });
        }
    }
}

fn main() {
    Runtime::new()
        .event(Duration::from_millis(800), ())
        .run(CounterNode::new())
        .unwrap()
}
