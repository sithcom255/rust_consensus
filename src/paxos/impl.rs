use crate::{Message, MessageType, Messages};
use std::collections::{HashMap, HashSet};

pub struct Proposer {
    // could be prime
    id: u32,
    // id to value
    states: HashMap<u64, BallotState>,
    acceptors: HashSet<u32>,
    learners: HashSet<u32>,
}

#[derive(Default)]
struct BallotState {
    ballot: u64,
    value: Option<u32>,
    remaining_promise: u32,
    remaining_confirm: u32,
    quorum: HashSet<u32>,
}

impl Proposer {
    fn new(
        id: u32,
        acceptors: HashSet<u32>,
        learners: HashSet<u32>,
    ) -> Result<Proposer, &'static str> {
        if acceptors.len() % 2 != 1 {
            return Err("Even number of acceptors passed, has to be odd");
        }

        Ok(Proposer {
            id,
            states: HashMap::new(),
            acceptors,
            learners,
        })
    }

    fn propose(&mut self, id: u64, value: u32) -> Option<Messages> {
        if !self.states.contains_key(&id) {
            return None;
        }

        let remaining_quorum = self.select_quorum();
        let targets: Vec<u32> = remaining_quorum.iter().copied().collect();
        self.states.insert(
            id,
            BallotState {
                ballot: id,
                value: Some(value),
                remaining_promise: targets.len() as u32,
                remaining_confirm: targets.len() as u32,
                quorum: remaining_quorum,
            },
        );

        let message = Message::Prepare {
            t: MessageType::Proposal,
            ballot: id,
        };

        Some(Messages { message, targets })
    }

    fn select_quorum(&self) -> HashSet<u32> {
        let quorum_size = self.acceptors.len() / 2 + 1;
        let mut quorum = HashSet::with_capacity(quorum_size);
        let mut iter = self.acceptors.iter();
        for _ in 0..quorum_size {
            if let Some(acceptor_id) = iter.next() {
                quorum.insert(*acceptor_id);
            }
        }

        quorum
    }

    fn process(&mut self, source: u32, promise: Message) -> Option<Messages> {
        // PROMISE
        if let Message::Promise {
            t,
            promised,
            ballot,
            max_ballot,
            value,
        } = promise
        {
            // REJECT FROM ONE
            if !promised {
                self.states.remove(&ballot);
                return None;
            }

            let mut remove: bool = false;

            if let Some(state) = self.states.get_mut(&ballot) {
                state.remaining_promise = state.remaining_promise - 1;

                if value.is_some() {
                    // UPDATE STATUS
                    if let Some(max_b) = max_ballot
                        && max_b > state.ballot
                    {
                        state.ballot = max_b;
                        state.value = value;
                    }
                }

                // LAST PROMISE
                if (state.remaining_promise == 0) {
                    // CLEANUP
                    remove = true;
                    // RESPOND
                    if let Some(ballot) = max_ballot {
                        let message = Message::Accept {
                            t: MessageType::Accept,
                            ballot: state.ballot,
                            value: state.value?,
                        };

                        return Some(Messages {
                            message,
                            targets: state.quorum.iter().copied().collect(),
                        });
                    }
                };
            }

            if remove {
                let max_ballot = self.states.remove(&ballot);
            }

            // LATE PROMISE
            return None;
        } else if let Message::Confirm { t, ballot, value } = promise {
            match self.states.get_mut(&ballot) {
                None => {
                    // THIS IS FOR ALREADY REMOVED
                }
                Some(state) => {
                    if state.remaining_promise > 0 {
                        // JUST DELETE
                    }

                    // IF VALUE != STATE.VALUE => JUST REMOVE
                    if value != state.value? {
                        // JUST DELETE
                    }

                    // ACCEPT
                    state.remaining_promise = state.remaining_promise - 1;
                    if state.remaining_promise == 0 {
                        let message = Message::Learn { ballot, value };

                        return Some(Messages {
                            message,
                            targets: self.learners.iter().copied().collect(),
                        });
                    }
                }
            };
        }

        None
    }
}

pub struct Acceptor {
    highest_ballot: Option<u64>,
    highest_ballot_value: Option<u32>,
}

impl Acceptor {
    // we promise:
    // not accepts any lower ballot that the incoming, (
    //  if higher ballot comes, accepts the higher number )
    // if we have lower ballot number with same value, accepts the message
    fn process(&mut self, message: Message) -> Option<Message> {
        if let Message::Prepare { t, ballot } = message {
            // reject lower
            if self.highest_ballot.is_some() && self.highest_ballot.unwrap() > ballot {
                return None;
            }

            if self.highest_ballot.is_none() {
                self.highest_ballot = Some(ballot);
            }

            return Some(Message::Promise {
                t: MessageType::Promise,
                promised: true,
                ballot,
                max_ballot: self.highest_ballot,
                value: self.highest_ballot_value,
            });
        }

        if let Message::Accept { t, ballot, value } = message {
            if !self.highest_ballot.is_none()
                || (self.highest_ballot.is_some() && self.highest_ballot.unwrap() < ballot)
            {
                self.highest_ballot = Some(ballot);
                self.highest_ballot_value = Some(value);

                return Some(Message::Confirm { t, ballot, value });
            }
        }

        None
    }
}

pub struct Learner {
    ballot: Option<u64>,
    value: Option<u32>,
}

impl Learner {
    fn process(&mut self, message: Message) {
        if let Message::Learn { ballot, value } = message {
            self.ballot = Some(ballot);
            self.value = Some(value);
        } else {
            panic!("Learner::process called on non-learner");
        }
    }
}
