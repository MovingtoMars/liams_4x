mod game_map;
mod map_position;
mod city_names;

use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Serialize, Deserialize};

pub use game_map::*;
pub use map_position::*;
pub use city_names::*;

pub const SERVER: &str = "127.0.0.1:12351";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToClientType {
    InitializeWorld(GameWorld),
    Event(GameEventType),
    Nothing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageToClient {
    pub message_type: MessageToClientType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToServerType {
    Hello,
    Action(GameActionType),
    NextTurn,
    Goodbye,
}

// Does this type add any value?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageToServer {
    pub message_type: MessageToServerType,
}

// TODO NAT hole-punching.
// TODO read up on for
pub struct Connection<S: Serialize, R: for<'a> Deserialize<'a> + Debug>  {
    stream: TcpStream,

    received_messages: Arc<Mutex<VecDeque<String>>>,

    _s: PhantomData<S>,
    _r: PhantomData<R>,
}

impl<S: Serialize, R: for<'a> Deserialize<'a> + Serialize + Debug> Connection<S, R> {
    pub fn new(stream: TcpStream) -> Self {

        let received_messages = Arc::new(Mutex::new(VecDeque::new()));

        let stream2 = stream.try_clone().unwrap();
        let received_messages2 = received_messages.clone();
        std::thread::spawn(move || {
            loop {
                let message: R = bincode::deserialize_from(&stream2).expect("bincode deserialization failed");
                // hack: can't get lifetimes to work nicely, so we serialize the message,
                // send it out of the thread, then deserialize it.
                received_messages2.lock().unwrap().push_back(ron::to_string(&message).unwrap());
            }
        });

        Self {
            stream,
            received_messages,
            _s: PhantomData,
            _r: PhantomData,
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub fn send_message(&mut self, message: S) {
        bincode::serialize_into(&self.stream, &message).expect("bincode serialization failed");
    }

    pub fn receive_message(&mut self) -> Option<R> {
        if let Some(received_message) = self.received_messages.lock().unwrap().pop_front() {
            let text: String = String::from_utf8(received_message.into()).unwrap();
            let message: R = ron::from_str(&text).unwrap();
            println!("Received: {:?}", message);

            Some(message)
        } else {
            None
        }
    }

    pub fn receive_message_blocking(&mut self) -> R {
        loop {
            if let Some(message) = self.receive_message() {
                return message;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
