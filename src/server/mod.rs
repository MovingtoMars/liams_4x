use crate::common::*;

use std::io::ErrorKind;
use std::net::TcpListener;
use std::time::Duration;

struct LobbyClient {
    connection: Connection<MessageToClient, MessageToServer>,
    name: String,
    is_host: bool,
    player_id: PlayerId,
    quitting: bool,
}

struct LobbyServer {
    listener: TcpListener,
    clients: Vec<LobbyClient>,
    player_id_generator: PlayerIdGenerator,
}

impl LobbyServer {
    fn new() -> Self {
        Self {
            listener: TcpListener::bind(SERVER_LISTEN).unwrap(),
            clients: Vec::new(),
            player_id_generator: PlayerIdGenerator::new(),
        }
    }

    fn handle_client_init(&mut self, mut connection: Connection<MessageToClient, MessageToServer>) -> LobbyClient {
        let name = if let MessageToServer { message_type: MessageToServerType::Hello { name } } = connection.receive_message_blocking() {
            name
        } else {
            panic!("unknown first message to server");
        };

        LobbyClient {
            connection,
            name,
            is_host: self.clients.is_empty(),
            player_id: self.player_id_generator.next(),
            quitting: false,
        }
    }

    fn try_accept(&mut self) -> Option<LobbyClient> {
        self.listener.set_nonblocking(true).unwrap();

        match self.listener.accept() {
            Ok((stream, _)) => Some(self.handle_client_init(Connection::new(stream))),
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    None
                } else {
                    panic!("{:?}", error);
                }
            }
        }
    }

    fn accept(&mut self) -> LobbyClient {
        self.listener.set_nonblocking(false).unwrap();

        match self.listener.accept() {
            Ok((stream, _)) => self.handle_client_init(Connection::new(stream)),
            Err(error) => panic!("{:?}", error),
        }
    }

    fn broadcast(&mut self, message: MessageToClient) {
        for client in &mut self.clients {
            client.connection.send_message(message.clone());
        }
    }

    fn player_names(&self) -> Vec<String> {
        self.clients.iter().map(|client| client.name.clone()).collect()
    }

    fn broadcast_player_names(&mut self) {
        self.broadcast(MessageToClient {
            message_type: MessageToClientType::PlayerListChanged(self.player_names()),
        });
    }

    fn start_game(mut self) -> GameServer {
        let init_players = self.clients.iter().map(|LobbyClient { player_id, name, .. }| {
            InitPlayer {
                id: *player_id,
                name: name.clone(),
            }
        }).collect();

        let game_world = GameWorld::generate(60, 40, init_players);

        for client in &mut self.clients {
            let initialize_stuff = MessageToClientType::InitializeWorld {
                world: game_world.clone(),
                player_id: client.player_id,
            };
            client.connection.send_message(MessageToClient { message_type: initialize_stuff.clone() });
        }

        GameServer {
            clients: self.clients.into_iter().map(|client| GameClient {
                connection: client.connection,
                name: client.name,
                is_host: client.is_host,
                player_id: client.player_id,
            }).collect(),
            game_world,
        }
    }

    pub fn run(mut self) -> Option<GameServer> {
        let host_client = self.accept();
        self.clients.push(host_client);
        self.broadcast_player_names();

        loop {
            if let Some(client) = self.try_accept() {
                self.clients.push(client);
                self.broadcast_player_names();
            }

            let mut start_game = false;

            for client in &mut self.clients {
                if let Some(message) = client.connection.receive_message() {
                    match message.message_type {
                        MessageToServerType::Start => {
                            if client.is_host {
                                start_game = true;
                            } else {
                                panic!();
                            }
                        }
                        MessageToServerType::Quit => {
                            // TODO should kick all the clients first.
                            if client.is_host {
                                self.broadcast(MessageToClient { message_type: MessageToClientType::Kick });
                                return None;
                            } else {
                                client.quitting = true;
                            }
                        }
                        _ => panic!("server received unexpected message: {:?}", message),
                    }
                }
            }

            let (quitting_clients, staying_clients) = self.clients.into_iter().partition(|client| client.quitting);
            self.clients = staying_clients;
            if !quitting_clients.is_empty() {
                self.broadcast_player_names();
            }

            if start_game {
                return Some(self.start_game());
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[allow(dead_code)]
struct GameClient {
    connection: Connection<MessageToClient, MessageToServer>,
    name: String,
    is_host: bool,
    player_id: PlayerId,
}

struct GameServer {
    clients: Vec<GameClient>,
    game_world: GameWorld,
}

impl GameServer {
    fn broadcast(&mut self, message: MessageToClient) {
        for client in &mut self.clients {
            client.connection.send_message(message.clone());
        }
    }

    fn run(&mut self) {
        loop {
            std::thread::sleep(Duration::from_millis(10));
            for i in 0..self.clients.len() {
                while let Some(message) = self.clients[i].connection.receive_message() {
                    match message.message_type {
                        MessageToServerType::Quit => {
                            // TODO clients will crash now
                            break;
                        }
                        MessageToServerType::Action(action) => {
                            let events = self.game_world.process_action(&action);
                            for event in events {
                                self.broadcast(MessageToClient { message_type: MessageToClientType::Event(event) })
                            }
                        }
                        MessageToServerType::NextTurn => {
                            let events = self.game_world.next_turn();
                            for event in events {
                                self.broadcast(MessageToClient { message_type: MessageToClientType::Event(event) })
                            }
                        }
                        MessageToServerType::Hello { .. } |
                        MessageToServerType::Start => panic!("Unexpected message: {:?}", message),
                    }
                }
            }
        }
    }
}

pub fn run_server() {
    println!("Server started.");
    if let Some(mut game_server) = LobbyServer::new().run() {
        game_server.run();
    }
    println!("Server stopped");
}
