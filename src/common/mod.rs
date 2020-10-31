mod game_map;

use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread::{yield_now, sleep};
use std::time::Duration;
use std::collections::VecDeque;
use std::time::Instant;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::TryRecvError;
use serde::{Serialize, Deserialize};
use laminar::*;

pub use game_map::*;

pub const SERVER: &str = "127.0.0.1:12351";
pub const CLIENT: &str = "127.0.0.1:12352";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToClientType {
    InitializeWorld(GameWorld),
    Event(GameEventType),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageToClient {
    pub message_type: MessageToClientType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToServerType {
    Hello,
    Action(GameActionType),
    Goodbye,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageToServer {
    pub message_type: MessageToServerType,
}

// TODO NAT hole-punching.
// TODO read up on for
pub struct Connection<S: Serialize, R: for<'a> Deserialize<'a> + Debug>  {
    target: SocketAddr,

    _s: PhantomData<S>,
    _r: PhantomData<R>,

    receiver: Receiver<SocketEvent>,
    sender: Sender<Packet>,
}

use std::net::ToSocketAddrs;

impl<S: Serialize, R: for<'a> Deserialize<'a> + Debug> Connection<S, R> {
    pub fn new<T: ToSocketAddrs, U: ToSocketAddrs>(bind: T, target: U) -> Self {
        let mut config = Config::default();
        config.heartbeat_interval = Some(Duration::from_millis(100));
        let mut socket = Socket::bind_with_config(bind, config).unwrap();

        let receiver = socket.get_event_receiver();
        let sender = socket.get_packet_sender();

        std::thread::spawn(move || {
            socket.start_polling_with_duration(Some(Duration::from_millis(10)));
        });

        Self {
            target: target.to_socket_addrs().unwrap().next().unwrap(),
            _s: PhantomData,
            _r: PhantomData,
            receiver,
            sender,
        }
    }

    pub fn send_message(&mut self, message: S) {
        self.sender.send(Packet::reliable_ordered(
            self.target,
            ron::to_string(&message).unwrap().into(),
            None,
        )).unwrap();
    }

    pub fn receive_message(&mut self) -> Option<R> {
        match self.receiver.try_recv() {
            Ok(SocketEvent::Packet(packet)) => {
                if packet.addr() == self.target {
                    let text: String = String::from_utf8(packet.payload().to_owned()).unwrap();
                    let message: R = ron::from_str(&text).unwrap();
                    println!("Received: {:?}", message);
                    Some(message)
                } else {
                    panic!("Client received packet from unknown sender");
                }
            }
            Ok(SocketEvent::Timeout(_)) => panic!("client connection to server timeout"),
            Ok(_) => None,
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("disconnected"),
        }
    }

    pub fn receive_message_blocking(&mut self) -> R {
        loop {
            if let Some(message) = self.receive_message() {
                return message;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
