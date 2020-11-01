mod imgui_wrapper;
mod utils;
mod constants;
mod drag;
mod hitbox;
mod object;

use std::collections::HashMap;

use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::DrawParam;
use ggez::graphics::Rect;
use ggez::graphics::{self, Image};
use ggez::mint;
use ggez::{Context, GameResult};

use imgui_wrapper::ImGuiWrapper;

use ncollide2d::math::Translation;

use crate::common::*;

use self::constants::*;
use self::utils::get_tile_window_pos;

use drag::Drag;
use object::ObjectType;
use hitbox::{Hitbox, get_hovered_object};

const SPRITE_TILE_BLANK: usize = 0;
const SPRITE_TILE_PLAINS: usize = 1;
const SPRITE_TILE_OCEAN: usize = 2;
const SPRITE_TILE_HIGHLIGHT: usize = 3;
const SPRITE_TILE_MOUNTAIN: usize = 4;

const SPRITE_CIVILIAN: usize = 5;
const SPRITE_SOLDIER: usize = 6;
const SPRITE_CIVILIAN_HIGHLIGHT: usize = 7;
const SPRITE_SOLDIER_HIGHLIGHT: usize = 8;


fn get_tile_image_src_rect(index: usize) -> Rect {
    let x = (index as f32) * 0.2 % 1.0;
    let y = ((index as f32) / 5.0).floor() / 5.0;

    Rect::new(x, y, 0.2, 0.2)
}

struct MainState {
    imgui_wrapper: ImGuiWrapper,
    hidpi_factor: f32,
    tile_sprites: Image,
    world: GameWorld,
    offset: Translation<f32>,
    current_drag: Option<Drag>,
    selected: Option<ObjectType>,
    // TODO turn this into a ncollide world
    hitboxes: HashMap<ObjectType, Hitbox>,
    connection: Connection<MessageToServer, MessageToClient>,
}

impl MainState {
    fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
        let mut connection: Connection<MessageToServer, MessageToClient> = Connection::new(CLIENT, SERVER);

        connection.send_message(MessageToServer { message_type: MessageToServerType::Hello });

        let world = match connection.receive_message_blocking().message_type {
            MessageToClientType::InitializeWorld(game_world) => game_world,
            message => panic!("Expected MessageToClientType::InitializeWorld, got {:?}", message),
        };

        let mut hitboxes = HashMap::new();
        for tile in world.map.tiles() {
            hitboxes.insert(
                ObjectType::Tile(tile.position),
                Hitbox::tile(get_tile_window_pos(tile.position)),
            );
        }
        for unit in world.units() {
            let window_pos = get_tile_window_pos(unit.position());
            let hitbox = match unit.unit_type() {
                UnitType::Civilian => Hitbox::civilian(window_pos),
                UnitType::Soldier => Hitbox::soldier(window_pos),
            };
            hitboxes.insert(
                ObjectType::Unit(unit.id()),
                hitbox,
            );
        }

        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let s = MainState {
            imgui_wrapper,
            hidpi_factor,
            tile_sprites: Image::new(ctx, "/sprites/tiles.png").unwrap(),
            world,
            offset: Translation::new(0.0, 0.0),
            current_drag: None,
            selected: None,
            hitboxes,
            connection,
        };
        Ok(s)
    }

    fn draw_tile(&self, ctx: &mut Context, tile: &Tile) {
        let sprite_index = match tile.tile_type {
            TileType::Plains => SPRITE_TILE_PLAINS,
            TileType::Ocean => SPRITE_TILE_OCEAN,
            TileType::Mountain => SPRITE_TILE_MOUNTAIN,
        };

        self.draw_tile_sprite(ctx, tile.position, sprite_index);
    }

    fn draw_tile_sprite(&self, ctx: &mut Context, pos: MapPosition, sprite_index: usize) {
        let dest_point = self.offset * get_tile_window_pos(pos);

        let params = DrawParam::default()
            .src(get_tile_image_src_rect(sprite_index))
            .dest(mint::Point2 { x: dest_point.x, y: dest_point.y });

        graphics::draw(ctx, &self.tile_sprites, params).unwrap();
    }

    fn send_action(&mut self, action: GameActionType) {
        self.connection.send_message(MessageToServer { message_type: MessageToServerType::Action(action) });
    }

    fn close_connection(&mut self) {
        self.connection.send_message(MessageToServer { message_type: MessageToServerType::Goodbye });
    }

    fn apply_event(&mut self, event: &GameEventType) {
        self.world.apply_event(&event);

        match *event {
            GameEventType::MoveUnit { unit_id, position } => {
                let object = ObjectType::Unit(unit_id);
                self.hitboxes.get_mut(&object).unwrap().set_tile_pos(position);
            }
            _ => {}
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        while let Some(MessageToClient { message_type }) = self.connection.receive_message() {
            match message_type {
                MessageToClientType::Event(event) => self.apply_event(&event),
                _ => panic!(),
            }
        }

        // TODO sleep enough to limit to 60 UPS
        // Possibly less? Could do with like 20
        std::thread::sleep(std::time::Duration::from_millis(20));

        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.close_connection();
        false
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        // Render game stuff
        {
            for tile in self.world.map.tiles() {
                self.draw_tile(ctx, tile);
            }

            for unit in self.world.units() {
                let sprite_index = match unit.unit_type() {
                    UnitType::Civilian => SPRITE_CIVILIAN,
                    UnitType::Soldier => SPRITE_SOLDIER,
                };
                self.draw_tile_sprite(ctx, unit.position(), sprite_index);
            }

            if let Some(selected) = self.selected {
                match selected {
                    ObjectType::Tile(pos) => {
                        self.draw_tile_sprite(ctx, pos, SPRITE_TILE_HIGHLIGHT);
                    }
                    ObjectType::City(city_id) => {
                        let pos = self.world.city(city_id).unwrap().position();
                        self.draw_tile_sprite(ctx, pos, SPRITE_TILE_HIGHLIGHT);
                    }
                    ObjectType::Unit(unit_id) => {
                        let unit = self.world.unit(unit_id).unwrap();
                        let sprite_index = match unit.unit_type() {
                            UnitType::Civilian => SPRITE_CIVILIAN_HIGHLIGHT,
                            UnitType::Soldier => SPRITE_SOLDIER_HIGHLIGHT,
                        };

                        self.draw_tile_sprite(ctx, unit.position(), sprite_index)
                    }
                }
            }
        }

        // Render game ui
        {
            use imgui::*;

            let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

            let MainState { world, selected, ref offset, connection, .. } = self;

            let func = |ui: &imgui::Ui| {
                const TURN_WINDOW_HEIGHT: f32 = 80.0;
                const TURN_WINDOW_WIDTH: f32 = 200.0;
                imgui::Window::new(im_str!("Turn"))
                    .size([TURN_WINDOW_WIDTH, TURN_WINDOW_HEIGHT], imgui::Condition::Always)
                    .position([0.0, screen_height - TURN_WINDOW_HEIGHT], imgui::Condition::Always)
                    .collapsible(false)
                    .movable(false)
                    .resizable(false)
                    .build(ui, || {
                        ui.text(format!("Turn {}", world.turn()));
                        let next_turn_clicked = ui.button(im_str!("Next turn"), [TURN_WINDOW_WIDTH - 20.0, 25.0]);
                        if next_turn_clicked {
                            connection.send_message(MessageToServer { message_type: MessageToServerType::NextTurn });
                        }
                    });

                if let Some(selected) = selected {
                    let text = match selected {
                        ObjectType::Tile(pos) => {
                            let tile_type = world.map.tile(*pos).tile_type;
                            format!("{} tile at {}", tile_type, pos)
                        },
                        ObjectType::Unit(unit_id) => {
                            let unit = world.unit(*unit_id).unwrap();
                            format!("{} at {}", unit.unit_type(), unit.position())
                        }
                        ObjectType::City(city_id) => {
                            let city = world.city(*city_id).unwrap();
                            format!("City: {} at {}", city.name(), city.position())
                        }
                    };

                    imgui::Window::new(im_str!("Selection"))
                      .size([400.0, screen_height], imgui::Condition::Always)
                      .position([screen_width - 400.0, 0.0], imgui::Condition::Always)
                      .collapsible(false)
                      .movable(false)
                      .resizable(false)
                      .build(ui, || {
                        ui.text(text);
                      });
                }

                for city in world.cities() {
                    let dest_point = offset * (Translation::new(0.0, 0.0) * get_tile_window_pos(city.position()));

                    let width = TILE_INNER_SMALL_WIDTH * 1.1;

                    imgui::Window::new(&ImString::new(format!("city for tile {}", city.position())))
                        .no_decoration()
                        // .size([50.0, 30.0], imgui::Condition::Always)
                        .position([dest_point.x + (TILE_WIDTH - width) / 2.0 - 5.0, dest_point.y], imgui::Condition::Always)
                        .always_auto_resize(true)
                        .draw_background(false)
                        .build(ui, || {
                            let clicked = ui.button(&ImString::new(city.name()), [width, 30.0]);
                            if clicked {
                                *selected = Some(ObjectType::City(city.id()));
                            }
                        });
                }
            };
            self.imgui_wrapper.render(ctx, self.hidpi_factor, func);
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
        if self.imgui_wrapper.want_capture_mouse() {
            return;
        }

        if let Some(ref mut drag) = self.current_drag {
            let (dx, dy) = drag.get_map_offset_delta(x, y);

            self.offset.x += dx;
            self.offset.y += dy;
        }
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down(button);
        if self.imgui_wrapper.want_capture_mouse() {
            return;
        }

        if let MouseButton::Left = button {
            self.current_drag = Some(Drag::new(x, y));
        }
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.imgui_wrapper.update_mouse_up(button);
        if self.imgui_wrapper.want_capture_mouse() {
            return;
        }

        let hovered = get_hovered_object(x, y, &self.offset, &self.hitboxes);

        if let MouseButton::Left = button {
            if let Some(ref drag) = self.current_drag {
                let click_occurred = !drag.passed_threshold();
                self.current_drag = None;
                if !click_occurred {
                    return;
                }
            }

            // A mouse click on the map occurred
            self.selected = hovered;
        } else if let MouseButton::Right = button {
            if let Some(ObjectType::Unit(unit_id)) = self.selected {
                if let Some(ObjectType::Tile(pos)) = hovered {
                    let action = GameActionType::MoveUnit { unit_id, position: pos };
                    self.send_action(action);
                }
            }
        }
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        self.imgui_wrapper.update_key_down(keycode, keymods);
        if self.imgui_wrapper.want_capture_keyboard() {
            return;
        }
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.imgui_wrapper.update_key_up(keycode, keymods);
        if self.imgui_wrapper.want_capture_keyboard() {
            return;
        }


        if [KeyCode::Q, KeyCode::Escape].contains(&keycode) {
            self.close_connection();
            ctx.continuing = false;
        }
    }

    fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
        self.imgui_wrapper.update_text(val);
        if self.imgui_wrapper.want_capture_keyboard() {
            return;
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
        println!("{:?}", graphics::screen_coordinates(ctx));
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.imgui_wrapper.update_scroll(x, y);
        if self.imgui_wrapper.want_capture_mouse() {
            return;
        }
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

    let state = &mut MainState::new(ctx, hidpi_factor).unwrap();

    event::run(ctx, event_loop, state).unwrap();
}
