use anyhow::{Result, Error};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{self, BufRead, StdoutLock, Write};

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
    pub rply: Option<usize>,
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

pub struct Sender<'a> {
    writer: StdoutLock<'a>,
    id: usize,
    node: String,
}

impl<'a> Sender<'a> {
    pub fn send(&mut self, dst: String, rply: Option<usize>, payload: impl Serialize) -> Result<()> {
        let message = Message {
            src: self.node.clone(),
            dst,
            body: Body {
                id: Some(self.id),
                rply,
                payload,
            }
        };

        self.id += 1;

        self.writer.write_all(&serde_json::to_vec(&message)?)?;
        self.writer.write_all(b"\n")?;

        Ok(())
    }
}

pub trait Node<Payload: DeserializeOwned + Serialize> {
    fn init(&mut self, init: Init) -> Result<()>;
    fn step(&self, message: Message<Payload>, sender: &mut Sender) -> Result<()>;

    fn run(&mut self) -> Result<()> {
        let mut stdin = io::stdin().lock().lines();
        let stdout = io::stdout().lock();

        let first = stdin.next().unwrap()?;
        let init_message: Message<InitPayload> = serde_json::from_str(&first)?;

        let InitPayload::Init(init) = init_message.body.payload else {
            return Err(Error::msg("bad init message"));
        };

        let mut sender = Sender {
            writer: stdout,
            id: 0,
            node: init.node_id.clone(),
        };

        self.init(init)?;

        sender.send(init_message.src, init_message.body.id, InitPayload::InitOk)?;

        while let Some(Ok(str)) = stdin.next() {
            let message = serde_json::from_str(&str)?;
            self.step(message, &mut sender)?;
        }

        Ok(())
    }
}
