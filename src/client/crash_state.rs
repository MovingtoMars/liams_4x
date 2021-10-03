use ggez::graphics;
use ggez::graphics::Rect;
use ggez::event::KeyCode;

use crate::client::scene::{Scene, SceneSwitch};

use super::InputEvent;
use super::SharedData;
use super::imgui_wrapper::ImGuiFonts;

pub struct CrashState {
    message: String,
}

impl CrashState {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }

    fn exit(&self) {
        panic!("{}", self.message.clone());
    }
}

impl Scene<SharedData, InputEvent> for CrashState {
    fn update(&mut self, _shared_data: &mut SharedData, _ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

        let Self { message } = self;

        let func = move |ui: &imgui::Ui, _fonts: &ImGuiFonts| {
            // ui.
            use imgui::*;

            imgui::Window::new(im_str!("Crash"))
                .position([0.0, 0.0], Condition::Always)
                .size([screen_width, screen_height], Condition::Always)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    ui.text(message);
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
            self.exit();
        }

        if let InputEvent::Quit = event {
            self.exit();
        }
    }

    fn name(&self) -> &str {
        "CrashState"
    }
}
