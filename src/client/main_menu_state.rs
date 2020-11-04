use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;
use ggez_goodies::scene::SceneSwitch;

use super::InputEvent;
use super::SharedData;
use super::imgui_wrapper::ImGuiFonts;
use super::lobby_state::LobbyState;

pub struct MainMenuState {
    // TODO merge these into 'what to do' enum?
    exiting_game: bool,
    quitting: bool,
    hosting: bool,
    joining: bool,
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

impl ggez_goodies::scene::Scene<SharedData, InputEvent> for MainMenuState {
    fn update(&mut self, _shared_data: &mut SharedData, ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        if self.quitting {
            ctx.continuing = false;
        }

        if self.hosting {
            return SceneSwitch::Push(Box::new(LobbyState::new(true, self.player_name.clone())));
        }

        if self.joining {
            return SceneSwitch::Push(Box::new(LobbyState::new(false, self.player_name.clone())));
        }

        if self.exiting_game {
            self.exiting_game = false;
        }

        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let Self { hosting, joining, quitting, player_name, .. } = self;

        let func = move |ui: &imgui::Ui, _fonts: &ImGuiFonts| {
            // ui.
            use imgui::*;

            let window_width = 600.0;
            let window_height = 400.0;

            let full_button_size: [f32; 2] = [window_width - ui.clone_style().window_padding[0] * 2.0, 40.0];

            imgui::Window::new(im_str!("Main Menu"))
                .position([(screen_width - window_width) / 2.0, (screen_height - window_height) / 2.0], Condition::Always)
                .size([window_width, window_height], Condition::Always)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut name_buf = ImString::new(player_name.as_str());
                    imgui::InputText::new(ui, im_str!("Player Name"), &mut name_buf)
                        .build();
                    *player_name = name_buf.to_str().into();

                    *hosting = ui.button(im_str!("Host Game"), full_button_size);
                    *joining = ui.button(im_str!("Join Game"), full_button_size);
                    *quitting = ui.button(im_str!("Quit"), full_button_size);
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
        "MainMenuState"
    }
}
