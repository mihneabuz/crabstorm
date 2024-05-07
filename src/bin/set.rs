use std::{collections::HashSet, time::Duration};

use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum SetPayload {
    Add { element: Value },
    AddOk,
    Read,
    ReadOk { value: HashSet<Value> },
    Replicate { elements: HashSet<Value> },
}

struct SetNode {
    id: String,
    neighbors: Vec<String>,
    set: HashSet<Value>,
}

impl SetNode {
    fn new() -> Self {
        Self {
            id: String::default(),
            neighbors: Vec::new(),
            set: HashSet::new(),
        }
    }
}

impl Node for SetNode {
    type Payload = SetPayload;
    type Event = ();

    fn init(&mut self, init: Init) {
        self.id = init.id;
        self.neighbors = init.nodes;
    }

    fn message(&mut self, message: Message<SetPayload>, sender: Sender<SetPayload>) {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            SetPayload::Add { element } => {
                self.set.insert(element);

                sender.send(dst, rply, SetPayload::AddOk);
            }

            SetPayload::Read => {
                let value = self.set.clone();

                sender.send(dst, rply, SetPayload::ReadOk { value });
            }

            SetPayload::Replicate { elements } => {
                self.set.extend(elements);
            }

            _ => unimplemented!(),
        };
    }

    fn event(&mut self, _: (), sender: Sender<SetPayload>) {
        for neigh in self.neighbors.iter() {
            let elements = self.set.clone();
            sender.send(neigh.clone(), None, SetPayload::Replicate { elements });
        }
    }
}

fn main() {
    Runtime::new()
        .event(Duration::from_secs(3), ())
        .run(SetNode::new())
        .unwrap()
}
