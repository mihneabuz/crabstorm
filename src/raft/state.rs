use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Clone, Debug)]
pub struct Topology {
    pub id: String,
    pub nodes: Vec<String>,
}

impl Topology {
    pub fn count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log<C> {
    pub term: u32,
    pub command: C,
}

#[derive(Clone, Debug)]
pub struct PersistentState<C> {
    pub term: u32,
    pub voted_for: Option<String>,

    pub commit_len: usize,
    pub log: Vec<Log<C>>,
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
    pub fn persist(&self) {}

    pub fn last_log_term(&self) -> Option<u32> {
        self.log.last().map(|last| last.term)
    }
}

#[derive(Clone, Debug)]
pub struct TransientState {
    pub role: Role,
    pub leader: Option<String>,

    pub votes_received: HashSet<String>,

    pub sent_len: HashMap<String, usize>,
    pub acked_len: HashMap<String, usize>,

    pub consumed: usize,
}

impl Default for TransientState {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            leader: None,
            votes_received: HashSet::new(),
            sent_len: HashMap::new(),
            acked_len: HashMap::new(),
            consumed: 0,
        }
    }
}
