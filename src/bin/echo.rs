use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crabstorm::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {}

impl EchoNode {
    fn new() -> Self {
        Self {}
    }
}

impl Node<EchoPayload> for EchoNode {
    fn oninit(&mut self, _: Init) -> Result<()> {
        Ok(())
    }

    fn onmessage(&mut self, message: Message<EchoPayload>, sender: &mut Sender) -> Result<()> {
        let EchoPayload::Echo{ echo } = message.body.payload else {
            return Err(Error::msg(format!("unexpected payload {:?}", message.body.payload)));
        };

        sender.send(message.src, message.body.id, EchoPayload::EchoOk { echo })?;

        Ok(())
    }
}

fn main() {
    Runtime::new().run(EchoNode::new()).unwrap()
}
