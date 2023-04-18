use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniquePayload {
    Generate,
    GenerateOk { id: String },
}

struct UniqueNode {}

impl UniqueNode {
    fn new() -> Self {
        Self {}
    }
}

impl Node<UniquePayload> for UniqueNode {
    fn oninit(&mut self, _: Init) -> Result<()> {
        Ok(())
    }

    fn onmessage(&mut self, message: Message<UniquePayload>, sender: &mut Sender) -> Result<()> {
        let UniquePayload::Generate = message.body.payload else {
            return Err(Error::msg(format!(
                "unexpected payload {:?}",
                message.body.payload
            )));
        };

        let id = Ulid::new().to_string();

        sender.send(
            message.src,
            message.body.id,
            UniquePayload::GenerateOk { id },
        )?;

        Ok(())
    }

    fn onevent(&mut self, _: (), _: &mut Sender) -> Result<()> {
        Ok(())
    }
}

fn main() {
    Runtime::new(UniqueNode::new()).run().unwrap()
}
