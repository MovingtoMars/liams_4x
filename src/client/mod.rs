mod imgui_wrapper;
mod utils;
mod constants;
mod drag;
mod hitbox;
mod selected_object;
mod in_game_state;

use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::{Context, GameResult};

use ggez_goodies::scene::SceneStack;

use self::in_game_state::InGameState;

pub struct SharedData {
    hidpi_factor: f32,
}

#[derive(Clone, Copy)]
pub enum InputEvent {
    MouseMotionEvent { x: f32, y: f32 },
    MouseDownEvent { button: MouseButton, x: f32, y: f32 },
    MouseUpEvent { button: MouseButton, x: f32, y: f32 },
    KeyDownEvent { code: KeyCode, mods: KeyMods },
    KeyUpEvent { code: KeyCode, mods: KeyMods },
    TextInputEvent(char),
    ScrollEvent { x: f32, y: f32 },
    Quit,
}

struct SceneStackHandler {
    scene_stack: SceneStack<SharedData, InputEvent>,
}

impl SceneStackHandler {
    fn new(ctx: &mut Context, hidpi_factor: f32) -> Self {
        let global_state = SharedData {
            hidpi_factor,
        };
        let mut scene_stack = SceneStack::new(ctx, global_state);

        scene_stack.push(Box::new(InGameState::new(ctx).unwrap()));

        Self { scene_stack }
    }

}

impl EventHandler for SceneStackHandler {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene_stack.update(ctx);
        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.scene_stack.input(InputEvent::Quit, true);
        false
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene_stack.draw(ctx);
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.scene_stack.input(InputEvent::MouseMotionEvent { x, y }, true);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.scene_stack.input(InputEvent::MouseDownEvent { button, x, y }, true);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.scene_stack.input(InputEvent::MouseUpEvent { button, x, y }, true);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        code: KeyCode,
        mods: KeyMods,
        _repeat: bool,
    ) {
        self.scene_stack.input(InputEvent::KeyDownEvent { mods, code }, true);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, code: KeyCode, mods: KeyMods) {
        self.scene_stack.input(InputEvent::KeyUpEvent { mods, code }, true);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
        self.scene_stack.input(InputEvent::TextInputEvent(val), true);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.scene_stack.input(InputEvent::ScrollEvent { x, y }, true);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
        println!("{:?}", graphics::screen_coordinates(ctx));
    }
}

pub fn run_client() {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    let title = "Liam's Civilization";

    let cb = ggez::ContextBuilder::new(title, "confused cyborg")
        .window_setup(
            conf::WindowSetup::default()
                .title(title)
                .vsync(true)
        )
        .window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(1400.0, 800.0)
                .min_dimensions(800.0, 600.0)
        )
        .add_resource_path(resource_dir);
    let (ref mut ctx, event_loop) = &mut cb.build().unwrap();

    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
    println!("main hidpi_factor = {}", hidpi_factor);

    let state = &mut SceneStackHandler::new(ctx, hidpi_factor);

    event::run(ctx, event_loop, state).unwrap();
}
