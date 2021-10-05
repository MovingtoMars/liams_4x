use std::net::{TcpStream, SocketAddr};

use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;
use imgui::ImString;
use crate::client::scene::{Scene, SceneSwitch};

use crate::client::{InputEvent, SharedData};
use crate::client::imgui_wrapper::ImGuiFonts;
use crate::client::scenes::lobby::LobbyState;

use crate::common::DEFAULT_SERVER;

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
            addr: ImString::new(DEFAULT_SERVER),
            quitting: false,
            addr_is_invalid: false,
            connection_failed: false,
        }
    }
}

impl Scene<SharedData, InputEvent> for InputServerAddrState {
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
        graphics::clear(ctx, graphics::Color::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let func = move |ui: &imgui::Ui, _fonts: &ImGuiFonts| {
            // ui.
            use imgui::*;

            let window_width = ui.current_font_size() * 22.0;
            let window_height = ui.current_font_size() * 16.0;

            let full_button_size: [f32; 2] = [window_width - ui.clone_style().window_padding[0] * 2.0, ui.current_font_size() * 2.0];

            imgui::Window::new(im_str!("Join Game"))
                .position([(screen_width - window_width) / 2.0, (screen_height - window_height) / 2.0], Condition::Always)
                .size([window_width, window_height], Condition::Always)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let input_changed = imgui::InputText::new(ui, im_str!("Server Address"), &mut self.addr)
                        .resize_buffer(true)
                        .build();

                    if input_changed {
                        self.addr_is_invalid = false;
                        self.connection_failed = false;
                    }

                    if self.addr_is_invalid {
                        ui.spacing();
                        ui.text("The address you entered is invalid.");
                        ui.spacing();
                    }

                    if self.connection_failed {
                        ui.spacing();
                        ui.text("Could not connect.");
                        ui.spacing();
                    }

                    self.joining = ui.button(im_str!("Join Game"), full_button_size);
                    self.quitting = ui.button(im_str!("Back"), full_button_size);
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
