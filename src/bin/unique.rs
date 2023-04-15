use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniquePayload {
    Generate,
    GenerateOk { id: usize },
}

struct UniqueNode {}

impl UniqueNode {
    fn new() -> Self {
        Self {}
    }
}

impl Node<UniquePayload> for UniqueNode {
    fn init(&mut self, init: Init) -> Result<()> {
        dbg!(init);
        todo!()
    }

    fn step(&self, message: Message<UniquePayload>, sender: &mut Sender) -> Result<()> {
        todo!()
    }
}

fn main() {
    UniqueNode::new().run().unwrap();
}
