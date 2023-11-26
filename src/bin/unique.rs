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

impl Node for UniqueNode {
    type Payload = UniquePayload;
    type Event = ();

    fn init(&mut self, _: Init) {}

    fn message(&mut self, message: Message<UniquePayload>, sender: Sender<UniquePayload>) {
        let UniquePayload::Generate = message.body.payload else {
            panic!("unexpected payload {:?}", message.body.payload);
        };

        let id = Ulid::new().to_string();

        sender.send(
            message.src,
            message.body.id,
            UniquePayload::GenerateOk { id },
        );
    }

    fn event(&mut self, _: (), _: Sender<UniquePayload>) {}
}

fn main() {
    Runtime::new().run(UniqueNode::new()).unwrap()
}
