mod draw;

use std::collections::HashMap;

use ggez::Context;
use ggez::GameResult;
use ggez::event::KeyCode;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::graphics::Image;
use imgui::ImString;
use ncollide2d::math::Translation;

use crate::client::scene::{Scene, SceneSwitch};

use crate::common::{
    Connection,
    GameActionType,
    GameEventType,
    GameWorld,
    MessageToClient,
    MessageToServer,
    PlayerId,
    Citizen,
};

use crate::client::InputEvent;
use crate::client::SharedData;
use crate::client::constants::*;
use super::crash::CrashState;
use crate::client::drag::Drag;
use crate::client::hitbox::{Hitbox, HitboxKey, get_hovered_object};
use crate::client::selected_object::SelectedObject;
use crate::client::utils::get_tile_window_pos;

const ZOOM_MIN: f32 = 0.5;
const ZOOM_MAX: f32 = 2.0;

pub struct InGameState {
    tile_sprites: Image,
    yield_sprites: Image,
    citizen_sprites: Image,
    world: GameWorld,
    offset: Translation<f32>,
    zoom: f32,
    mouse_x: f32,
    mouse_y: f32,
    current_drag: Option<Drag>,
    selected: Option<SelectedObject>,
    // TODO turn this into a ncollide world
    hitboxes: HashMap<HitboxKey, Hitbox>,
    connection: Connection<MessageToServer, MessageToClient>,
    drawable_window_size: (f32, f32),
    player_id: PlayerId,
    quitting: bool,
    crash: Option<String>,
    display_tech_tree: bool,
}

impl InGameState {
    pub fn new(ctx: &mut Context, world: GameWorld, player_id: PlayerId, connection: Connection<MessageToServer, MessageToClient>) -> GameResult<Self> {
        let mut hitboxes = HashMap::new();
        for tile in world.map.tiles() {
            hitboxes.insert(
                HitboxKey::Tile(tile.position),
                Hitbox::tile(tile.position),
            );
        }
        for unit in world.units() {
            hitboxes.insert(
                HitboxKey::Unit(unit.id()),
                Hitbox::unit(unit.position(), unit.unit_type()),
            );
        }

        let offset = {
            // Center the camera on the first init we own
            let my_civ = world.player(player_id).unwrap().civilization_id();
            let my_position = world.units().find(|unit| unit.owner() == my_civ).unwrap().position();

            let window_pos = get_tile_window_pos(my_position);
            let draw_size = graphics::drawable_size(ctx);
            Translation::new(
                -window_pos.x + draw_size.0 / 2.0 - TILE_WIDTH / 2.0,
                -window_pos.y + draw_size.1 / 2.0 - TILE_HEIGHT / 2.0,
            )
        };

        let s = InGameState {
            tile_sprites: Image::new(ctx, "/sprites/tiles.png").unwrap(),
            yield_sprites: Image::new(ctx, "/sprites/yields.png").unwrap(),
            citizen_sprites: Image::new(ctx, "/sprites/citizens.png").unwrap(),
            world,
            offset,
            zoom:1.0,
            mouse_x:0.0,
            mouse_y:0.0,
            current_drag: None,
            selected: None,
            hitboxes,
            connection,
            drawable_window_size: (0.0, 0.0),
            player_id,
            quitting: false,
            crash: None,
            display_tech_tree: false,
        };
        Ok(s)
    }

    fn send_action(&mut self, action: GameActionType) {
        self.connection.send_message(MessageToServer::Action(action));
    }

    fn on_quit(&mut self) {
        self.connection.send_message(MessageToServer::Quit);
    }

    fn apply_event(&mut self, event: &GameEventType) {
        self.world.apply_event(&event);

        match *event {
            GameEventType::MoveUnit { unit_id, position, .. } => {
                let key = HitboxKey::Unit(unit_id);
                self.hitboxes.get_mut(&key).unwrap().set_tile_pos(position);
            }
            GameEventType::DeleteUnit { unit_id } => {
                let key = HitboxKey::Unit(unit_id);
                self.hitboxes.remove(&key);
                if let Some(SelectedObject::Unit(selected_unit_id)) = self.selected {
                    if selected_unit_id == unit_id {
                        self.selected = None;
                    }
                }
            }
            GameEventType::FoundCity { position, owner } => {
                if owner != self.world.player(self.player_id).unwrap().civilization_id() {
                    return;
                }
                let city_id = self.world.map.tile(position).city.unwrap();
                let city_name = self.world.city(city_id).unwrap().name();
                self.selected = Some(SelectedObject::City(city_id, ImString::new(city_name)));
            }
            GameEventType::NewUnit { ref template, position, unit_id, .. } => {
                self.hitboxes.insert(
                    HitboxKey::Unit(unit_id),
                    Hitbox::unit(position, template.unit_type),
                );
            }
            GameEventType::Crash { ref message } => {
                self.crash = Some(message.clone());
            }
            _ => {}
        }
    }

    fn can_control_unit(&self, unit: &crate::common::Unit) -> bool {
        self.world.player(self.player_id).unwrap().civilization_id() == unit.owner()
    }
}

impl Scene<SharedData, InputEvent> for InGameState {
    fn update(&mut self, _shared_data: &mut SharedData, _ctx: &mut ggez::Context) -> SceneSwitch<SharedData, InputEvent> {
        if let Some(message) = self.crash.take() {
            return SceneSwitch::Replace(Box::new(CrashState::new(message)));
        }

        if self.quitting {
            self.on_quit();
            return SceneSwitch::Pop;
        }

        while let Some(message) = self.connection.receive_message() {
            match message {
                MessageToClient::Event(event) => self.apply_event(&event),
                _ => panic!(),
            }
        }

        // TODO sleep enough to limit to 60 UPS
        // Possibly less? Could do with like 20
        std::thread::yield_now();

        SceneSwitch::None
    }

    fn draw(&mut self, shared_data: &mut SharedData, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.drawable_window_size = graphics::drawable_size(ctx);

        graphics::clear(ctx, graphics::Color::BLACK);

        // Render game stuff
        {
            self.draw_tiles(ctx);
            self.draw_rivers(ctx);
            self.draw_cities_borders(ctx);
            self.draw_units(ctx);
            self.draw_selected_highlight(ctx);
            self.draw_tiles_yields(ctx);
        }

        // Render game ui
        {
            //  TODO use ui.current_font_size() to size buttons, etc

            let rc = shared_data.imgui_wrapper.render_start(ctx, shared_data.hidpi_factor);

            self.draw_general_sidebar_ui(ctx, &rc);
            self.draw_selected_sidebar_ui(ctx, &rc);
            self.draw_cities_ui(ctx, &rc);
            if self.display_tech_tree {
                self.draw_tech_tree_ui(ctx, &rc);
            }

            rc.render(ctx);
        }

        graphics::present(ctx)
    }

    fn input(&mut self, shared_data: &mut SharedData, event: InputEvent, _started: bool) {
        if shared_data.imgui_wrapper.handle_event(&event) {
            return;
        }

        match event {
            InputEvent::MouseMotionEvent { x, y } => {
                if let Some(ref mut drag) = self.current_drag {
                    let (dx, dy) = drag.get_map_offset_delta(x, y, self.zoom);

                    self.offset.x += dx;
                    self.offset.y += dy;
                }
                self.mouse_x = x;
                self.mouse_y = y;
            }
            InputEvent::MouseDownEvent { button, x, y } => {
                if let MouseButton::Left = button {
                    self.current_drag = Some(Drag::new(x, y));
                }
            }
            InputEvent::MouseUpEvent { button, x, y } => {
                let mut hitboxes = self.hitboxes.clone();
                if let Some(SelectedObject::City(city_id, _)) = self.selected {
                    let city = self.world.city(city_id).unwrap();
                    let tiles: Vec<_> = city.territory_tiles().map(|pos| *pos).filter(|pos| *pos != city.position()).collect();
                    for pos in tiles {
                        hitboxes.insert(HitboxKey::Citizen(pos), Hitbox::citizen(pos));
                    }
                }
                let hovered = get_hovered_object(x, y, self.zoom, &self.offset, &hitboxes);

                if let MouseButton::Left = button {
                    if let Some(ref drag) = self.current_drag {
                        let click_occurred = !drag.passed_threshold();
                        self.current_drag = None;
                        if !click_occurred {
                            return;
                        }
                    }

                    // A mouse click on the map occurred
                    if let Some(hovered) = hovered {
                        match hovered {
                            HitboxKey::Tile(position) => {
                                self.selected = Some(SelectedObject::Tile(position));
                            }
                            HitboxKey::Unit(unit_id) => {
                                self.selected = Some(SelectedObject::Unit(unit_id));
                            }
                            HitboxKey::Citizen(position) => {
                                if let Some(SelectedObject::City(city_id, _)) = self.selected {
                                    let city = self.world.city(city_id).unwrap();
                                    let citizen = city.territory().get(&position).unwrap();
                                    let locked = match citizen {
                                        Some(Citizen::Normal) | None => true,
                                        Some(Citizen::Locked) => false,
                                    };

                                    self.send_action(GameActionType::SetCitizenLocked { city_id, position, locked });
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                    };
                } else if let MouseButton::Right = button {
                    if let Some(SelectedObject::Unit(unit_id)) = self.selected {
                        if let Some(HitboxKey::Tile(pos)) = hovered {
                            let sp = self.world.map.shortest_path(self.world.unit(unit_id).unwrap().position(),pos).unwrap();
                            println!("{:?}", sp);

                            let action = GameActionType::MoveUnit { unit_id, position: pos };
                            self.send_action(action);
                        }
                    }
                }
            }
            InputEvent::KeyDownEvent { code: _code, mods: _mods } => {

            }
            InputEvent::KeyUpEvent { code, mods: _mods } => {
                if [KeyCode::Escape].contains(&code) {
                    self.quitting = true;
                }
            }
            InputEvent::TextInputEvent(_val) => {

            }
            InputEvent::ScrollEvent { x: _x, y } => {
                let mut factor = 1.1_f32.powf(y);
                let old_zoom = self.zoom;
                self.zoom *= factor;

                if self.zoom < ZOOM_MIN {
                    factor *= ZOOM_MIN/self.zoom;
                    self.zoom = ZOOM_MIN;
                }

                if self.zoom > ZOOM_MAX {
                    factor *= ZOOM_MAX/self.zoom;
                    self.zoom = ZOOM_MAX;
                }

                let x_shift = (self.mouse_x - self.mouse_x / factor) / old_zoom;
                let y_shift = (self.mouse_y - self.mouse_y / factor) / old_zoom;

                // adjust offset so map at cursor is stationary
                self.offset.x -= x_shift;
                self.offset.y -= y_shift;
            }
            InputEvent::Quit => {
                self.quitting = true;
            }
        }
    }

    fn name(&self) -> &str {
        "InGameState"
    }
}
