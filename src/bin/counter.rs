use anyhow::Result;
use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum CounterPayload {
    Add { delte: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
}

struct CounterNode {}

impl CounterNode {
    fn new() -> Self {
        Self {}
    }
}

impl Node<CounterPayload> for CounterNode {
    fn oninit(&mut self, _: Init) -> Result<()> {
        Ok(())
    }

    fn onmessage(&mut self, message: Message<CounterPayload>, sender: &mut Sender) -> Result<()> {
        Ok(())
    }

    fn onevent(&mut self, _: (), _: &mut Sender) -> Result<()> {
        Ok(())
    }
}

fn main() {
    Runtime::new(CounterNode::new()).run().unwrap()
}
