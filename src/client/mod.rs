mod imgui_wrapper;
mod utils;
mod constants;
mod drag;
mod hitbox;
mod selected_object;

use std::collections::HashMap;

use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::DrawParam;
use ggez::graphics::Rect;
use ggez::graphics::{self, Image};
use ggez::mint;
use ggez::{Context, GameResult};

use imgui::ImString;
use imgui_wrapper::ImGuiWrapper;

use ncollide2d::math::Translation;

use crate::common::*;

use self::constants::*;
use self::imgui_wrapper::ImGuiFonts;
use self::utils::get_tile_window_pos;

use drag::Drag;
use selected_object::SelectedObject;
use hitbox::{Hitbox, HitboxKey, get_hovered_object};

const SPRITE_TILE_BLANK: usize = 0;
const SPRITE_TILE_PLAINS: usize = 1;
const SPRITE_TILE_OCEAN: usize = 2;
const SPRITE_TILE_HIGHLIGHT: usize = 3;
const SPRITE_TILE_MOUNTAIN: usize = 4;

const SPRITE_CIVILIAN: usize = 5;
const SPRITE_SOLDIER: usize = 6;
const SPRITE_CIVILIAN_HIGHLIGHT: usize = 7;
const SPRITE_SOLDIER_HIGHLIGHT: usize = 8;

const SPRITE_TILE_HIGHLIGHT_BLUE_1: usize = 10;
const SPRITE_TILE_HIGHLIGHT_BLUE_2: usize = 11;
const SPRITE_TILE_HIGHLIGHT_BLUE_3: usize = 12;


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
    selected: Option<SelectedObject>,
    // TODO turn this into a ncollide world
    hitboxes: HashMap<HitboxKey, Hitbox>,
    connection: Connection<MessageToServer, MessageToClient>,
    drawable_window_size: (f32, f32),
    civilization_id: CivilizationId,
}

impl MainState {
    fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
        let mut connection: Connection<MessageToServer, MessageToClient> = Connection::new(std::net::TcpStream::connect(SERVER).unwrap());

        connection.send_message(MessageToServer { message_type: MessageToServerType::Hello { name: "devplayer".into() } });

        connection.receive_message_blocking();

        let (world, civilization_id) = match connection.receive_message_blocking().message_type {
            MessageToClientType::InitializeWorld { world, civilization_id } => (world, civilization_id),
            message => panic!("Expected MessageToClientType::InitializeWorld, got {:?}", message),
        };

        let mut hitboxes = HashMap::new();
        for tile in world.map.tiles() {
            hitboxes.insert(
                HitboxKey::Tile(tile.position),
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
                HitboxKey::Unit(unit.id()),
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
            drawable_window_size: (0.0, 0.0),
            civilization_id,
        };
        Ok(s)
    }

    fn draw_tile(&self, ctx: &mut Context, tile: &Tile) {
        let sprite_index = match tile.tile_type {
            TileType::Plains => SPRITE_TILE_PLAINS,
            TileType::Ocean => SPRITE_TILE_OCEAN,
            TileType::Mountain => SPRITE_TILE_MOUNTAIN,
        };

        self.draw_tile_sprite(ctx, tile.position, sprite_index, None);
    }

    fn in_drawable_bounds(&self, dest_point: mint::Point2<f32>, width: f32, height: f32) -> bool {
        dest_point.x < self.drawable_window_size.0 + width &&
        dest_point.y < self.drawable_window_size.1 + height &&
        dest_point.x > -width &&
        dest_point.y > -height
    }

    fn draw_tile_sprite(&self, ctx: &mut Context, pos: MapPosition, sprite_index: usize, color: Option<graphics::Color>) {
        let dest_point = self.offset * get_tile_window_pos(pos);
        let dest_point = mint::Point2 { x: dest_point.x, y: dest_point.y };

        if !self.in_drawable_bounds(dest_point, TILE_WIDTH, TILE_HEIGHT) {
            return;
        }

        let mut params = DrawParam::default()
            .src(get_tile_image_src_rect(sprite_index))
            .dest(dest_point);

        if let Some(color) = color {
            params = params.color(color);
        }

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
                if owner != self.civilization_id {
                    return;
                }
                let city_id = self.world.map.tile(position).city.unwrap();
                let city_name = self.world.city(city_id).unwrap().name();
                self.selected = Some(SelectedObject::City(city_id, ImString::new(city_name)));
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
        std::thread::yield_now();

        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.close_connection();
        false
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.drawable_window_size = graphics::drawable_size(ctx);

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
                let color = if unit.remaining_movement() > 0 {
                    None
                } else {
                    Some(graphics::Color::new(0.7, 0.7, 0.7, 0.9))
                };
                self.draw_tile_sprite(ctx, unit.position(), sprite_index, color);
            }

            if let Some(ref selected) = self.selected {
                match selected {
                    SelectedObject::Tile(pos) => {
                        self.draw_tile_sprite(ctx, *pos, SPRITE_TILE_HIGHLIGHT, None);
                    }
                    SelectedObject::City(city_id, _) => {
                        let pos = self.world.city(*city_id).unwrap().position();
                        self.draw_tile_sprite(ctx, pos, SPRITE_TILE_HIGHLIGHT, None);
                    }
                    SelectedObject::Unit(unit_id) => {
                        let unit = self.world.unit(*unit_id).unwrap();
                        let position = unit.position();
                        let sprite_index = match unit.unit_type() {
                            UnitType::Civilian => SPRITE_CIVILIAN_HIGHLIGHT,
                            UnitType::Soldier => SPRITE_SOLDIER_HIGHLIGHT,
                        };

                        self.draw_tile_sprite(ctx, position, sprite_index, None);

                        if ggez::input::mouse::button_pressed(ctx, MouseButton::Right) {
                            let neighbor_map = position.neighbors_at_distance(
                                self.world.map.width(),
                                self.world.map.height(),
                                unit.remaining_movement(),
                                true,
                            );

                            for (neighbor, distance) in neighbor_map {
                                let sprite_index = match distance {
                                    0 => continue,
                                    1 => SPRITE_TILE_HIGHLIGHT_BLUE_1,
                                    2 => SPRITE_TILE_HIGHLIGHT_BLUE_2,
                                    _ => SPRITE_TILE_HIGHLIGHT_BLUE_3,
                                };
                                self.draw_tile_sprite(ctx, neighbor, sprite_index, None);
                            }
                        }
                    }
                }
            }
        }

        // Render game ui
        {
            use imgui::*;

            let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

            let MainState { civilization_id: you_civilization_id, world, selected, ref offset, connection, .. } = self;

            let fps = ggez::timer::fps(ctx);

            //  TODO use ui.current_font_size() to size buttons, etc
            let func = |ui: &imgui::Ui, fonts: &ImGuiFonts| {
                imgui::Window::new(im_str!("Overview"))
                    .position([0.0, 0.0], imgui::Condition::Once)
                    .size_constraints([200.0, 50.0], [10000000.0, 1000000.0])
                    .always_auto_resize(true)
                    .build(ui, || {
                        ui.text(format!("FPS: {:.0}", fps));
                        ui.text(format!("World: {}x{}", world.map.width(), world.map.height()));
                        ui.spacing();
                        ui.separator();
                        ui.spacing();
                        ui.text("Civilizations:");
                        for civ in world.civilizations() {
                            let you_str = if civ.id() == *you_civilization_id { "(you)" } else { "" };
                            ui.text(format!("{} {}", civ.player_name(), you_str));
                        }
                    });

                const TURN_WINDOW_HEIGHT: f32 = 110.0;
                const TURN_WINDOW_WIDTH: f32 = 250.0;
                imgui::Window::new(im_str!("Turn"))
                    .size([TURN_WINDOW_WIDTH, TURN_WINDOW_HEIGHT], imgui::Condition::Always)
                    .position([0.0, screen_height - TURN_WINDOW_HEIGHT], imgui::Condition::Always)
                    .collapsible(false)
                    .movable(false)
                    .resizable(false)
                    .build(ui, || {
                        ui.text(format!("Turn {}", world.turn()));
                        let open_sans_semi_bold_30_handle = ui.push_font(fonts.open_sans_semi_bold_30);
                        let next_turn_clicked = ui.button(im_str!("Next turn"), [TURN_WINDOW_WIDTH - 20.0, 40.0]);
                        open_sans_semi_bold_30_handle.pop(ui);
                        if next_turn_clicked {
                            connection.send_message(MessageToServer { message_type: MessageToServerType::NextTurn });
                        }
                    });

                if let Some(selected) = selected {
                    const SIDEBAR_WIDTH: f32 = 350.0;
                    let sidebar_button_size: [f32; 2] = [SIDEBAR_WIDTH - ui.clone_style().window_padding[0] * 2.0, 40.0];
                    imgui::Window::new(im_str!("Selection"))
                        .size([SIDEBAR_WIDTH, screen_height], imgui::Condition::Always)
                        .position([screen_width - SIDEBAR_WIDTH, 0.0], imgui::Condition::Always)
                        .collapsible(false)
                        .movable(false)
                        .resizable(false)
                        .build(ui, || {
                            match selected {
                                SelectedObject::Tile(pos) => {
                                    let tile_type = world.map.tile(*pos).tile_type;
                                    ui.text(format!("{} tile at {}", tile_type, pos));
                                },
                                SelectedObject::Unit(unit_id) => {
                                    let unit = world.unit(*unit_id).unwrap();
                                    let owner_name = world.civilization(unit.owner()).unwrap().player_name();
                                    ui.text(format!("{} at {}", unit.unit_type(), unit.position()));
                                    ui.text(format!("Owner: {}", owner_name));
                                    ui.text(format!("Movement: {}/{}", unit.remaining_movement(), unit.total_movement()));
                                    ui.spacing();
                                    ui.spacing();
                                    ui.separator();
                                    ui.spacing();
                                    ui.spacing();
                                    ui.spacing();

                                    if unit.has_settle_ability() {
                                        // TODO disable button when can't settle
                                        let founding_city = ui.button(im_str!("Found city"), sidebar_button_size);
                                        if founding_city {
                                            let action = GameActionType::FoundCity { unit_id: *unit_id };
                                            connection.send_message(MessageToServer { message_type: MessageToServerType::Action(action) });
                                        }
                                    }
                                }
                                SelectedObject::City(city_id, ref mut city_name_buf) => {
                                    let city = world.city(*city_id).unwrap();
                                    // TODO name length limit?
                                    let city_name_changed = ui.input_text(im_str!(""), city_name_buf)
                                        .resize_buffer(true)
                                        // If the user holds down a key, it can send quite a lot of data.
                                        // Perhaps debounce, or set this to false.
                                        .enter_returns_true(false)
                                        .build();

                                    let owner_name = world.civilization(city.owner()).unwrap().player_name();
                                    ui.text(format!("Owner: {}", owner_name));
                                    ui.text(format!("City at {}", city.position()));

                                    if city_name_changed {
                                        let action = GameActionType::RenameCity { city_id: *city_id, name: city_name_buf.to_string() };
                                        connection.send_message(MessageToServer { message_type: MessageToServerType::Action(action) });
                                    }
                                }
                            }
                        });
                }


                let open_sans_semi_bold_30_handle = ui.push_font(fonts.open_sans_semi_bold_30);
                for city in world.cities() {
                    let dest_point = offset * (Translation::new(0.0, 0.0) * get_tile_window_pos(city.position()));

                    let width = TILE_INNER_WIDTH * 1.1;

                    imgui::Window::new(&ImString::new(format!("city for tile {}", city.position())))
                        .no_decoration()
                        // .size([50.0, 30.0], imgui::Condition::Always)
                        .position([dest_point.x + (TILE_WIDTH - width) / 2.0 - 5.0, dest_point.y], imgui::Condition::Always)
                        .always_auto_resize(true)
                        .draw_background(false)
                        .build(ui, || {
                            let clicked = ui.button(&ImString::new(city.name()), [width, 40.0]);
                            if clicked {
                                *selected = Some(SelectedObject::City(city.id(), ImString::new(city.name())));
                            }
                        });
                }
                open_sans_semi_bold_30_handle.pop(ui);
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
            self.selected = hovered.map(|hovered| match hovered {
                HitboxKey::Tile(position) => SelectedObject::Tile(position),
                HitboxKey::Unit(unit_id) => SelectedObject::Unit(unit_id),
            });
        } else if let MouseButton::Right = button {
            if let Some(SelectedObject::Unit(unit_id)) = self.selected {
                if let Some(HitboxKey::Tile(pos)) = hovered {
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


        if [KeyCode::Escape].contains(&keycode) {
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
