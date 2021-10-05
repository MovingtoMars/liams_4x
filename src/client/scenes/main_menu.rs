use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;
use crate::client::scene::{Scene, SceneSwitch};

use crate::client::InputEvent;
use crate::client::SharedData;
use super::lobby::LobbyState;
use super::input_server_addr::InputServerAddrState;

pub struct MainMenuState {
    // TODO merge these into 'what to do' enum?
    exiting_game: bool,
    quitting: bool,
    hosting: bool,
    joining: bool,
    // TODO maybe move this to shared state
    player_name: String,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            exiting_game: false,
            quitting: false,
            hosting: false,
            joining: false,
            player_name: "devplayer".into(),
        }
    }
}

impl Scene<SharedData, InputEvent> for MainMenuState {
    fn update(&mut self, _shared_data: &mut SharedData, ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        if self.quitting {
            ctx.continuing = false;
        }

        if self.hosting {
            return SceneSwitch::Push(Box::new(LobbyState::new(None, self.player_name.clone())));
        }

        if self.joining {
            return SceneSwitch::Push(Box::new(InputServerAddrState::new(self.player_name.clone())));
        }

        if self.exiting_game {
            self.exiting_game = false;
        }

        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let render_context = shared_data.imgui_wrapper.render_start(ctx, shared_data.hidpi_factor);
        let ui = &render_context.ui;

        use imgui::*;

        let window_width = ui.current_font_size() * 22.0;
        let window_height = ui.current_font_size() * 16.0;

        let full_button_size: [f32; 2] = [window_width - ui.clone_style().window_padding[0] * 2.0, ui.current_font_size() * 2.0];

        imgui::Window::new(im_str!("Main Menu"))
            .position([(screen_width - window_width) / 2.0, (screen_height - window_height) / 2.0], Condition::Always)
            .size([window_width, window_height], Condition::Always)
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                let mut name_buf = ImString::new(self.player_name.as_str());
                imgui::InputText::new(&ui, im_str!("Player Name"), &mut name_buf)
                    .resize_buffer(true)
                    .build();
                self.player_name = name_buf.to_str().into();

                self.hosting = ui.button(im_str!("Host Game"), full_button_size);
                self.joining = ui.button(im_str!("Join Game"), full_button_size);
                self.quitting = ui.button(im_str!("Quit"), full_button_size);
            });

        render_context.render(ctx);

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
        "MainMenuState"
    }
}
