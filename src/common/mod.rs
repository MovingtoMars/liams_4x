mod game_map;
mod map_position;
mod city_names;
mod civilization;
mod unit;
mod generate_world;
mod player;
mod resource;
mod tile;
mod city;
mod yields;
mod actions;
mod events;
mod building;
mod tech;
mod game_world;

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
pub use civilization::*;
pub use unit::*;
pub use generate_world::*;
pub use player::*;
pub use resource::*;
pub use tile::*;
pub use city::*;
pub use yields::*;
pub use actions::*;
pub use events::*;
pub use building::*;
pub use tech::*;
pub use game_world::*;

pub const SERVER_LISTEN: &str = "0.0.0.0:12351";
pub const DEFAULT_SERVER: &str = "127.0.0.1:12351";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyInfo {
    pub players: Vec<(String, PlayerId)>,
    pub you: PlayerId,
    pub host: PlayerId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToClient {
    InitializeWorld{ world: GameWorld, player_id: PlayerId },
    Event(GameEventType),
    LobbyInfo(LobbyInfo),
    Kick,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageToServer {
    Hello { name: String },
    Start,
    Action(GameActionType),
    Quit,
}

// TODO NAT hole-punching.
// TODO read up on for
pub struct Connection<S: Serialize, R: serde::de::DeserializeOwned + 'static + Debug + Send>  {
    stream: TcpStream,

    received_messages: Arc<Mutex<VecDeque<R>>>,

    _s: PhantomData<S>,
    _r: PhantomData<R>,
}

impl<S: Serialize, R: serde::de::DeserializeOwned + 'static + Debug + Send> Connection<S, R> {
    pub fn new(stream: TcpStream) -> Self {

        let received_messages = Arc::new(Mutex::new(VecDeque::new()));

        let stream2 = stream.try_clone().unwrap();
        let received_messages2 = received_messages.clone();
        std::thread::spawn(move || {
            loop {
                let message: R = bincode::deserialize_from(&stream2).expect("bincode deserialization failed");
                received_messages2.lock().unwrap().push_back(message);
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

    // TODO this can probably take &S
    pub fn send_message(&mut self, message: S) {
        bincode::serialize_into(&self.stream, &message).expect("bincode serialization failed");
    }

    pub fn receive_message(&mut self) -> Option<R> {
        if let Some(received_message) = self.received_messages.lock().unwrap().pop_front() {
            // println!("Received: {:?}", received_message);

            Some(received_message)
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
