use serde::{Deserialize, Serialize};

use super::Log;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rpc<C> {
    pub term: u32,
    pub payload: RpcType<C>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RpcType<C> {
    VoteRequest(VoteRequest),
    VoteResponse(VoteResponse),
    AppendRequest(AppendRequest<C>),
    AppendResponse(AppendResponse),
    ForwardRequest(ForwardRequest<C>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteRequest {
    pub candidate: String,
    pub last_log_index: usize,
    pub last_log_term: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteResponse {
    pub voter: String,
    pub granted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendRequest<C> {
    pub leader: String,
    pub prefix_len: usize,
    pub prefix_term: u32,
    pub commit_len: usize,
    pub suffix: Vec<Log<C>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendResponse {
    pub follower: String,
    pub ack: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForwardRequest<C> {
    pub follower: String,
    pub command: C,
}
