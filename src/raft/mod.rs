mod rpc;
mod state;
mod timer;

use std::fmt::Debug;

pub use rpc::*;
use state::*;
use timer::*;

pub struct Raft<C> {
    topology: Topology,
    persistent: PersistentState<C>,
    transient: TransientState,
    timer: Timer,
}

#[derive(Clone, Debug)]
pub enum Delivery<C> {
    Unicast(String, Rpc<C>),
    Broadcast(Rpc<C>),
    Multicast(Vec<(String, Rpc<C>)>),
}

impl<C> Raft<C>
where
    C: Clone + Debug,
{
    pub fn new(id: String, nodes: Vec<String>) -> Self {
        Self {
            topology: Topology { id, nodes },
            persistent: PersistentState::default(),
            transient: TransientState::default(),
            timer: Timer::new(1000, 1000),
        }
    }

    pub fn id(&self) -> &String {
        &self.topology.id
    }

    pub fn nodes(&self) -> impl Iterator<Item = &String> {
        self.topology.nodes.iter()
    }

    pub fn others(&self) -> impl Iterator<Item = &String> {
        self.topology.nodes.iter().filter(|node| *node != self.id())
    }

    pub fn log(&self) -> &Vec<Log<C>> {
        &self.persistent.log
    }

    pub fn is_leader(&self) -> bool {
        self.transient.role == Role::Leader
    }

    pub fn apply(&mut self, command: C) -> Option<Delivery<C>> {
        let term = self.persistent.term;

        if self.is_leader() {
            self.persistent.log.push(Log { term, command });
            self.persistent.persist();

            self.transient
                .acked_len
                .insert(self.id().clone(), self.persistent.log.len());

            Some(self.on_replicate())
        } else {
            let leader = self.transient.leader.clone()?;
            if &leader == self.id() {
                return None;
            }

            Some(Delivery::Unicast(
                leader.to_string(),
                Rpc {
                    term: self.persistent.term,
                    payload: RpcType::ForwardRequest(ForwardRequest {
                        follower: self.id().clone(),
                        command,
                    }),
                },
            ))
        }
    }

    pub fn consume(&mut self) -> Option<C> {
        if self.transient.consumed < self.persistent.commit_len {
            self.transient.consumed += 1;
            Some(
                self.persistent.log[self.transient.consumed - 1]
                    .command
                    .clone(),
            )
        } else {
            None
        }
    }

    pub fn tick(&mut self) -> Option<Delivery<C>> {
        if self.is_leader() {
            return Some(self.on_replicate());
        }

        if self.timer.expired() {
            let message = self.on_timeout();
            return Some(Delivery::Broadcast(message));
        }

        None
    }

    pub fn process(&mut self, from: String, rpc: Rpc<C>) -> Option<Delivery<C>> {
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
                let message = self.on_append_request(rpc.term, request);
                Some(Delivery::Unicast(from, message))
            }

            RpcType::AppendResponse(response) => {
                self.on_append_response(rpc.term, response);
                None
            }

            RpcType::ForwardRequest(request) => self.apply(request.command),
        }
    }

    fn on_replicate(&self) -> Delivery<C> {
        let mut messages = Vec::new();

        for node in self.topology.nodes.iter() {
            if node == self.id() {
                continue;
            }

            let prefix_len = self.transient.sent_len.get(node).copied().unwrap_or(0);
            let suffix = self.persistent.log[prefix_len..].to_vec();

            let prefix_term = if prefix_len > 0 {
                self.persistent.log[prefix_len - 1].term
            } else {
                0
            };

            messages.push((
                node.clone(),
                Rpc {
                    term: self.persistent.term,
                    payload: RpcType::AppendRequest(AppendRequest {
                        leader: self.id().clone(),
                        prefix_len,
                        prefix_term,
                        commit_len: self.persistent.commit_len,
                        suffix,
                    }),
                },
            ));
        }

        Delivery::Multicast(messages)
    }

    fn on_timeout(&mut self) -> Rpc<C> {
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

    fn on_vote_request(&mut self, term: u32, request: VoteRequest) -> Rpc<C> {
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
                }
            }
        }
    }

    fn on_append_request(&mut self, term: u32, request: AppendRequest<C>) -> Rpc<C> {
        if term > self.persistent.term {
            self.persistent.term = term;
            self.persistent.voted_for = None;
        }

        self.timer.reset();

        if self.persistent.term == term {
            self.transient.role = Role::Follower;
            self.transient.leader = Some(request.leader);
        }

        let term_ok = term == self.persistent.term;

        let log_ok = (self.persistent.log.len() >= request.prefix_len)
            && (request.prefix_len == 0
                || self.persistent.log[request.prefix_len - 1].term == request.prefix_term);

        let ack = if term_ok && log_ok {
            let suffix_len = request.suffix.len();
            self.append_commands(request.prefix_len, request.commit_len, request.suffix);
            Some(request.prefix_len + suffix_len)
        } else {
            None
        };

        self.persistent.persist();

        Rpc {
            term,
            payload: RpcType::AppendResponse(AppendResponse {
                follower: self.id().clone(),
                ack,
            }),
        }
    }

    fn append_commands(&mut self, prefix: usize, commit: usize, suffix: Vec<Log<C>>) {
        if suffix.is_empty() && self.persistent.log.len() > prefix {
            let index = self.persistent.log.len().min(prefix + suffix.len()) - 1;
            if self.persistent.log[index].term != suffix[index - prefix].term {
                self.persistent.log.truncate(prefix);
            }
        }

        if prefix + suffix.len() > self.persistent.log.len() {
            let range = self.persistent.log.len() - prefix..suffix.len();
            let mut suffix = suffix;
            self.persistent.log.extend(suffix.drain(range));
        }

        if commit > self.persistent.commit_len {
            self.persistent.commit_len = commit;
        };

        self.persistent.persist();
    }

    fn on_append_response(&mut self, term: u32, response: AppendResponse) {
        if term > self.persistent.term {
            self.persistent.term = term;
            self.persistent.voted_for = None;
            self.persistent.persist();

            self.transient.role = Role::Follower;
            self.timer.reset();

            return;
        }

        if term == self.persistent.term && self.transient.role == Role::Leader {
            if let Some(ack) = response.ack {
                let entry = self
                    .transient
                    .acked_len
                    .entry(response.follower.clone())
                    .or_default();

                if ack >= *entry {
                    self.transient
                        .sent_len
                        .insert(response.follower.clone(), ack);

                    *entry = ack;

                    self.commit_commands();
                }
            } else {
                self.transient
                    .sent_len
                    .entry(response.follower)
                    .and_modify(|sent| *sent = sent.checked_sub(1).unwrap_or(0));
            }
        }
    }

    fn commit_commands(&mut self) {
        let mut commit = self.persistent.commit_len;

        while commit < self.persistent.log.len() {
            let mut acks = 0;

            for node in self.topology.nodes.iter() {
                if self.transient.acked_len.get(node).copied().unwrap_or(0) > commit {
                    acks += 1;
                }
            }

            if acks >= (self.topology.count() + 1) / 2 {
                commit += 1;
            } else {
                break;
            }
        }

        self.persistent.commit_len = commit;
        self.persistent.persist();
    }
}
