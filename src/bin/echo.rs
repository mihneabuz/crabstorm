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

impl Node for EchoNode {
    type Payload = EchoPayload;
    type Event = ();

    fn init(&mut self, _: Init) {}

    fn message(&mut self, message: Message<EchoPayload>, sender: Sender<EchoPayload>) {
        let EchoPayload::Echo { echo } = message.body.payload else {
            panic!("unexpected payload {:?}", message.body.payload);
        };

        sender.send(message.src, message.body.id, EchoPayload::EchoOk { echo });
    }
}

fn main() {
    Runtime::new().run(EchoNode::new()).unwrap()
}
