use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
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
}

struct CounterNode {
    id: String,
    acc: usize,
    others: HashMap<String, usize>,
}

impl CounterNode {
    fn new() -> Self {
        Self {
            id: "".to_string(),
            acc: 0,
            others: HashMap::new(),
        }
    }
}

impl Node<CounterPayload> for CounterNode {
    fn oninit(&mut self, init: Init) -> Result<()> {
        self.id = init.node_id;
        self.others.extend(
            init.node_ids
                .into_iter()
                .filter(|n| *n != self.id)
                .map(|n| (n, 0)),
        );
        Ok(())
    }

    fn onmessage(&mut self, message: Message<CounterPayload>, sender: &mut Sender) -> Result<()> {
        let dst = message.src;
        let rply = message.body.id;

        match message.body.payload {
            CounterPayload::Add { delta } => {
                self.acc += delta;
                sender.send(dst, rply, CounterPayload::AddOk)?;
            }

            CounterPayload::Read => {
                let value = self.acc + self.others.values().sum::<usize>();
                sender.send(dst, rply, CounterPayload::ReadOk { value })?;
            }

            CounterPayload::Gossip { value } => {
                self.others
                    .entry(dst)
                    .and_modify(|acc| *acc = cmp::max(value, *acc));
            }

            _ => unimplemented!(),
        };

        Ok(())
    }

    fn onevent(&mut self, _: (), sender: &mut Sender) -> Result<()> {
        for n in self.others.keys() {
            sender.send(n.clone(), None, CounterPayload::Gossip { value: self.acc })?;
        }

        Ok(())
    }
}

fn main() {
    Runtime::new(CounterNode::new())
        .event(Duration::from_millis(800), ())
        .run()
        .unwrap()
}
