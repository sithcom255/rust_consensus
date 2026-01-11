mod paxos;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::FromBytesUntilNulError;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, Read, Write};
use std::net::{AddrParseError, Shutdown, SocketAddr};
use std::os::fd::IntoRawFd;
use std::time::Duration;

fn main() {
    println!("Hello, world!");
}

#[repr(u16)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
enum MessageType {
    Connect = 1,
    Ack = 2,
    Proposal = 3,
    Promise = 4,
    Accept = 5,
}

impl TryFrom<u16> for MessageType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MessageType::Connect),
            2 => Ok(MessageType::Ack),
            3 => Ok(MessageType::Proposal),
            _ => Err(()),
        }    }
}

impl From<&BytesMut> for MessageType {
    fn from(bytes: &BytesMut) -> Self {
        MessageType::try_from(u16::from_be_bytes([bytes[0], bytes[1]])).unwrap()
    }
}

impl Into<u16> for MessageType {
    fn into(self) -> u16 {
        self as u16
    }
}

struct Messages {
    message: Message,
    targets: Vec<u32>
}

#[derive(Debug)]
pub enum Message {
    Connect {
        t: MessageType,
    },
    Ack {
        t: MessageType,
    },
    Prepare {
        t: MessageType,
        ballot: u64,
    },
    Promise {
        t: MessageType,
        promised: bool,
        ballot: u64,
        max_ballot: Option<u64>,
        value: Option<u32>
    },
    Accept {
        t: MessageType,
        ballot: u64,
        value: u32,
    },
    Confirm {
        t: MessageType,
        ballot: u64,
        value: u32,
    },
    Learn {
        ballot: u64,
        value: u32
    },
}

impl Message {
    fn from_bytes(bytes: &mut BytesMut, messages: &mut Vec<Message>) -> Option<usize> {
        if bytes.len() < 6 {
            return None;
        }

        let t = MessageType::from(&*bytes);
        let data_size = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);

        if bytes.len() < 6 + data_size as usize {
            return None;
        }

        bytes.advance(6);
        bytes.chunk();

        let m = match t {
            MessageType::Connect => Some(Message::Connect { t }),
            MessageType::Ack => Some(Message::Ack { t }),
            MessageType::Proposal => None,
            MessageType::Promise => None,
            MessageType::Accept => None,
        };

        if let Some(m) = m {
            messages.push(m);
        }

        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ProtocolMessage {
    field: usize,
}

#[test]

fn hello_world() {
    let x: [u8; 2] = [1, 2];
    let mut bytes = BytesMut::with_capacity(6);
    bytes.put_u16(MessageType::Connect as u16);
    bytes.put_u32(0);

    let mut vec1: Vec<Message> = vec![];
    Message::from_bytes(&mut bytes, &mut vec1);

    println!("{:?}", vec1);
}
