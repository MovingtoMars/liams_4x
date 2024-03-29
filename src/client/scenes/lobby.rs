use std::thread;
use std::time::Duration;
use std::net::TcpStream;

use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;
use crate::client::scene::{Scene, SceneSwitch};

use crate::client::InputEvent;
use crate::client::SharedData;
use crate::client::imgui_wrapper::ImGuiFonts;
use crate::client::scenes::in_game::InGameState;
use crate::common::{DEFAULT_SERVER, Connection, LobbyInfo, MessageToClient, MessageToServer};
use crate::server;

pub struct LobbyState {
    quitting_from_lobby: bool,
    // TODO force shut down server if it does not go gracefully.
    hosting: bool,
    connection: Option<Connection<MessageToServer, MessageToClient>>,
    starting_game: bool,
    lobby_info: Option<LobbyInfo>,
}

fn start_server() {
    thread::spawn(|| server::run_server());
    // TODO HACK: wait for server to start listening.
    thread::sleep(Duration::from_millis(100));
}

impl LobbyState {
    pub fn new(joining: Option<TcpStream>, player_name: String) -> Self {
        let mut connection;
        let hosting;

        if let Some(stream) = joining {
            connection = Connection::new(stream);
            connection.send_message(MessageToServer::Hello { name: player_name.clone() });
            hosting = false;
        } else {
            start_server();
            // We assume that this won't fail...
            connection = Connection::new(std::net::TcpStream::connect(DEFAULT_SERVER).unwrap());
            connection.send_message(MessageToServer::Hello { name: player_name.clone() });
            hosting = true;
        };

        Self {
            quitting_from_lobby: false,
            starting_game: false,
            hosting,
            connection: Some(connection),
            lobby_info: None,
        }
    }
}

impl Scene<SharedData, InputEvent> for LobbyState {
    fn update(&mut self, _shared_data: &mut SharedData, ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        if self.quitting_from_lobby {
            self.connection.as_mut().unwrap().send_message(MessageToServer::Quit);
            return SceneSwitch::Pop;
        }

        if self.starting_game {
            self.starting_game = false;
            self.connection.as_mut().unwrap().send_message(MessageToServer::Start);
        }

        if let Some(connection) = &mut self.connection {
            if let Some(message) = connection.receive_message() {
                match message {
                    MessageToClient::LobbyInfo(lobby_info) => {
                        self.lobby_info = Some(lobby_info);
                    }
                    MessageToClient::InitializeWorld { world, player_id } => {
                        return SceneSwitch::Push(Box::new(InGameState::new(ctx, world, player_id, self.connection.take().unwrap()).unwrap()));
                    }
                    MessageToClient::Kick => {
                        return SceneSwitch::Pop;
                    }
                    _ => panic!("client in lobby state received unexpected message: {:?}", message),
                }
            }
        } else {
            panic!("no connection in lobby");
        }

        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let func = |ui: &imgui::Ui, _fonts: &ImGuiFonts| {
            use imgui::*;

            let window_width = ui.current_font_size() * 22.0;
            let window_height = ui.current_font_size() * 16.0;

            let full_button_size: [f32; 2] = [window_width - ui.clone_style().window_padding[0] * 2.0, ui.current_font_size() * 2.0];

            imgui::Window::new(im_str!("Lobby"))
                .position([(screen_width - window_width) / 2.0, (screen_height - window_height) / 2.0], Condition::Always)
                .size([window_width, window_height], Condition::Always)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    if self.hosting {
                        ui.text(format!("Hosting as: {}", DEFAULT_SERVER));
                        ui.spacing();
                        self.starting_game = ui.button(im_str!("Start Game"), full_button_size);
                    } else {
                        ui.text(format!("Connected to: {}", self.connection.as_ref().unwrap().peer_addr()));
                    }

                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    ui.text("Players:");

                    if let Some(lobby_info) = &self.lobby_info {
                        for (player_name, player_id) in &lobby_info.players {
                            let mut text = player_name.to_owned();
    
                            if *player_id == lobby_info.host {
                                text = text + " (host)";
                            }
                            if *player_id == lobby_info.you {
                                text = text + " (you)";
                            }
                            ui.text(text);
                        }
                    }

                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    self.quitting_from_lobby = ui.button(im_str!("Main Menu"), full_button_size);
                });
        };

        shared_data.imgui_wrapper.render(ctx, shared_data.hidpi_factor, func);

        graphics::present(ctx)
    }

    fn input(&mut self, shared_data: &mut SharedData, event: InputEvent, _started: bool) {
        if shared_data.imgui_wrapper.handle_event(&event) {
            return;
        }

        if let InputEvent::KeyUpEvent { code: KeyCode::Escape, .. } = event {
            self.quitting_from_lobby = true;
        }
    }

    fn name(&self) -> &str {
        "LobbyState"
    }
}
