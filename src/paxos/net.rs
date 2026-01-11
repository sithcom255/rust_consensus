use std::collections::hash_map::Values;
use crate::paxos::r#impl::{Acceptor, Learner, Proposer};
use crate::{Message, Messages};

struct Network {
    proposers: Vec<Proposer>,
    acceptors: Vec<Acceptor>,
    learners: Vec<Learner>,
    to_send_messages: Vec<Messages>,
}

impl Network {

    pub fn propose(&mut self, i: usize, ballot:u64, values: u32) {
    }

    pub fn process(&mut self) {
    }
}
