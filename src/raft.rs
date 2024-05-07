use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Clone, Debug)]
struct Topology {
    id: String,
    nodes: Vec<String>,
}

impl Topology {
    fn count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Clone, Debug)]
struct Log<C> {
    term: u32,
    command: C,
}

#[derive(Clone, Debug)]
pub struct PersistentState<C> {
    term: u32,
    voted_for: Option<String>,

    commit_len: usize,
    log: Vec<Log<C>>,
}

impl<C> Default for PersistentState<C> {
    fn default() -> Self {
        Self {
            term: 0,
            voted_for: None,
            commit_len: 0,
            log: Vec::new(),
        }
    }
}

impl<C> PersistentState<C> {
    fn persist(&self) {}

    fn load(&mut self) {}

    fn last_log_term(&self) -> Option<u32> {
        self.log.last().map(|last| last.term)
    }
}

#[derive(Clone, Debug)]
struct TransientState {
    role: Role,
    leader: Option<String>,

    votes_received: HashSet<String>,

    sent_len: HashMap<String, usize>,
    acked_len: HashMap<String, usize>,
}

impl Default for TransientState {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            leader: None,
            votes_received: HashSet::new(),
            sent_len: HashMap::new(),
            acked_len: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct Timer {
    last: Instant,
    timeout: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(500 + random::<u64>() % 500),
            last: Instant::now(),
        }
    }
}

impl Timer {
    fn expired(&self) -> bool {
        self.last.elapsed() > self.timeout
    }

    fn reset(&mut self) {
        *self = Self::default()
    }

    fn tick(&mut self) {
        self.last = Instant::now();
    }
}

pub struct Raft {
    topology: Topology,
    persistent: PersistentState<()>,
    transient: TransientState,
    timer: Timer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rpc {
    term: u32,
    payload: RpcType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RpcType {
    VoteRequest(VoteRequest),
    VoteResponse(VoteResponse),
    AppendRequest(AppendRequest),
    AppendResponse(AppendResponse),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteRequest {
    candidate: String,
    last_log_index: usize,
    last_log_term: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteResponse {
    voter: String,
    granted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendRequest {
    leader: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendResponse {
    follower: String,
    ack: bool,
}

#[derive(Clone, Debug)]
pub enum Delivery {
    Broadcast(Rpc),
    Unicast(String, Rpc),
}

impl Raft {
    pub fn new(id: String, nodes: Vec<String>) -> Self {
        Self {
            topology: Topology { id, nodes },
            persistent: PersistentState::default(),
            transient: TransientState::default(),
            timer: Timer::default(),
        }
    }

    pub fn id(&self) -> &String {
        &self.topology.id
    }

    pub fn nodes(&self) -> &Vec<String> {
        &self.topology.nodes
    }

    pub fn is_leader(&self) -> bool {
        self.transient.role == Role::Leader
    }

    pub fn tick(&mut self) -> Option<Delivery> {
        eprintln!("leader: {:?}", self.transient.leader);

        if self.timer.expired() {
            let message = self.on_timeout();
            return Some(Delivery::Broadcast(message));
        }

        if self.is_leader() {
            return Some(self.on_heartbeat());
        }

        None
    }

    pub fn process(&mut self, from: String, rpc: Rpc) -> Option<Delivery> {
        match rpc.payload {
            RpcType::VoteRequest(request) => {
                let message = self.on_vote_request(rpc.term, request);
                Some(Delivery::Unicast(from, message))
            }

            RpcType::VoteResponse(response) => {
                self.on_vote_response(rpc.term, response);
                None
            }

            RpcType::AppendRequest(request) => {
                let message = self.on_append_requst(rpc.term, request);
                Some(Delivery::Unicast(from, message))
            }

            RpcType::AppendResponse(response) => {
                self.on_append_response(rpc.term, response);
                None
            }
        }
    }

    fn on_heartbeat(&self) -> Delivery {
        Delivery::Broadcast(Rpc {
            term: self.persistent.term,
            payload: RpcType::AppendRequest(AppendRequest {
                leader: self.id().clone(),
            }),
        })
    }

    fn on_timeout(&mut self) -> Rpc {
        self.persistent.term += 1;
        self.persistent.voted_for = Some(self.id().clone());

        self.persistent.persist();

        self.transient.role = Role::Candidate;
        self.transient.votes_received.insert(self.id().clone());

        let last_log_index = self.persistent.log.len();
        let last_log_term = self.persistent.last_log_term().unwrap_or(0);

        let message = Rpc {
            term: self.persistent.term,
            payload: RpcType::VoteRequest(VoteRequest {
                candidate: self.id().clone(),
                last_log_index,
                last_log_term,
            }),
        };

        self.timer.reset();

        message
    }

    fn on_vote_request(&mut self, term: u32, request: VoteRequest) -> Rpc {
        if term > self.persistent.term {
            self.persistent.term = term;
            self.persistent.voted_for = None;

            self.transient.role = Role::Follower;
        }

        let last_index = self.persistent.log.len();
        let last_term = self.persistent.last_log_term().unwrap_or(0);

        let term_ok = term == self.persistent.term;

        let log_ok = request.last_log_term > last_term
            || (request.last_log_term == last_term && request.last_log_index >= last_index);

        let vote_ok = self.persistent.voted_for.is_none()
            || self
                .persistent
                .voted_for
                .as_ref()
                .is_some_and(|vote| *vote == request.candidate);

        let granted = if term_ok && log_ok && vote_ok {
            self.persistent.voted_for = Some(request.candidate);
            true
        } else {
            false
        };

        self.persistent.persist();

        Rpc {
            term,
            payload: RpcType::VoteResponse(VoteResponse {
                voter: self.id().clone(),
                granted,
            }),
        }
    }

    fn on_vote_response(&mut self, term: u32, response: VoteResponse) {
        if term > self.persistent.term {
            self.persistent.term = term;
            self.persistent.voted_for = None;
            self.persistent.persist();

            self.transient.role = Role::Follower;
            self.timer.reset();

            return;
        }

        if response.granted
            && term == self.persistent.term
            && self.transient.role == Role::Candidate
        {
            self.transient.votes_received.insert(response.voter);

            if self.transient.votes_received.len() >= (self.topology.count() + 1) / 2 {
                self.transient.role = Role::Leader;
                self.transient.leader = Some(self.id().clone());

                self.timer.reset();

                for node in self.topology.nodes.iter() {
                    self.transient
                        .sent_len
                        .insert(node.clone(), self.persistent.log.len());
                    self.transient.acked_len.insert(node.clone(), 0);

                    // replicate_log
                }
            }
        }
    }

    fn on_append_requst(&mut self, term: u32, request: AppendRequest) -> Rpc {
        if term > self.persistent.term {
            self.persistent.term = term;
            self.persistent.voted_for = None;
        }

        self.timer.tick();

        if self.persistent.term == term {
            self.transient.role = Role::Follower;
            self.transient.leader = Some(request.leader);
        }

        self.persistent.persist();

        Rpc {
            term,
            payload: RpcType::AppendResponse(AppendResponse {
                follower: self.id().clone(),
                ack: true,
            }),
        }
    }

    fn on_append_response(&mut self, term: u32, response: AppendResponse) {}
}
