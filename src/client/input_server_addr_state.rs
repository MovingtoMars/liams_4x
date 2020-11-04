use std::net::{TcpStream, SocketAddr};

use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;
use ggez_goodies::scene::SceneSwitch;
use imgui::ImString;

use super::InputEvent;
use super::SharedData;
use super::imgui_wrapper::ImGuiFonts;
use super::lobby_state::LobbyState;

use crate::common::SERVER;

pub struct InputServerAddrState {
    player_name: String,
    joining: bool,
    addr: ImString,
    quitting: bool,
    addr_is_invalid: bool,
    connection_failed: bool,
}

impl InputServerAddrState {
    pub fn new(player_name: String) -> Self {
        Self {
            player_name,
            joining: false,
            addr: ImString::new(SERVER),
            quitting: false,
            addr_is_invalid: false,
            connection_failed: false,
        }
    }
}

impl ggez_goodies::scene::Scene<SharedData, InputEvent> for InputServerAddrState {
    fn update(&mut self, _shared_data: &mut SharedData, _ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        if self.joining {
            if let Ok(parsed_addr) = self.addr.to_str().parse::<SocketAddr>() {
                match TcpStream::connect(parsed_addr) {
                    Ok(stream) => {
                        return SceneSwitch::Push(Box::new(LobbyState::new(Some(stream), self.player_name.clone())));
                    }
                    Err(_) => {
                        self.connection_failed = true;
                    }
                }
            } else {
                self.addr_is_invalid = true;
            }
            self.joining = false;
        }

        if self.quitting {
            return SceneSwitch::Pop;
        }

        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let Self { joining, quitting, addr, addr_is_invalid, connection_failed, .. } = self;

        let func = move |ui: &imgui::Ui, _fonts: &ImGuiFonts| {
            // ui.
            use imgui::*;

            let window_width = 600.0;
            let window_height = 400.0;

            let full_button_size: [f32; 2] = [window_width - ui.clone_style().window_padding[0] * 2.0, 40.0];

            imgui::Window::new(im_str!("Join Game"))
                .position([(screen_width - window_width) / 2.0, (screen_height - window_height) / 2.0], Condition::Always)
                .size([window_width, window_height], Condition::Always)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let input_changed = imgui::InputText::new(ui, im_str!("Server Address"), addr)
                        .resize_buffer(true)
                        .build();

                    if input_changed {
                        *addr_is_invalid = false;
                        *connection_failed = false;
                    }

                    if *addr_is_invalid {
                        ui.spacing();
                        ui.text("The address you entered is invalid.");
                        ui.spacing();
                    }

                    if *connection_failed {
                        ui.spacing();
                        ui.text("Could not connect.");
                        ui.spacing();
                    }

                    *joining = ui.button(im_str!("Join Game"), full_button_size);
                    *quitting = ui.button(im_str!("Back"), full_button_size);
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
            self.quitting = true;
        }
    }

    fn name(&self) -> &str {
        "InputServerAddrState"
    }
}
