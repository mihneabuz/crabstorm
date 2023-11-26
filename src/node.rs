use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub reply: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node {
    type Payload;
    type Event;

    fn init(&mut self, init: Init);
    fn message(&mut self, message: Message<Self::Payload>, sender: Sender<Self::Payload>);
    fn event(&mut self, event: Self::Event, sender: Sender<Self::Payload>);
}

pub struct Sender<P> {
    inner: flume::Sender<(String, Option<usize>, P)>,
}

impl<P> Sender<P> {
    pub(crate) fn new(sender: flume::Sender<(String, Option<usize>, P)>) -> Self {
        Self { inner: sender }
    }

    pub fn send(&self, dest: String, reply: Option<usize>, payload: P) {
        self.inner
            .send((dest, reply, payload))
            .expect("Failed to send message");
    }
}
