use std::collections::HashMap;

use ggez::Context;
use ggez::GameResult;
use ggez::event::KeyCode;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::mint;
use imgui::ImString;
use ncollide2d::math::Translation;

use ggez_goodies::scene::SceneSwitch;

use crate::common::{
    Connection,
    GameActionType,
    GameEventType,
    GameWorld,
    TilePosition,
    MessageToClient,
    MessageToServer,
    PlayerId,
    Tile,
    TileType,
    UnitType,
    CivilizationId,
    TileEdge,
    ResourceType,
    Yields,
    Vegetation,
};

use super::InputEvent;
use super::SharedData;
use super::constants::*;
use super::crash_state::CrashState;
use super::drag::Drag;
use super::hitbox::{Hitbox, HitboxKey, get_hovered_object};
use super::imgui_wrapper::ImGuiFonts;
use super::selected_object::SelectedObject;
use super::utils::get_tile_window_pos;

fn get_tile_image_src_rect(index: usize) -> Rect {
    get_image_src_rect(index, 5, 8)
}

fn get_yield_image_src_rect(index: usize) -> Rect {
    get_image_src_rect(index, 5, 5)
}

fn get_image_src_rect(index: usize, x_count: usize, y_count: usize) -> Rect {
    let x = (index as f32) / x_count as f32 % 1.0;
    let y = ((index as f32) / x_count as f32).floor() / y_count as f32;

    Rect::new(x, y, 1.0 / x_count as f32, 1.0 / y_count as f32)
}

pub struct InGameState {
    tile_sprites: Image,
    yield_sprites: Image,
    world: GameWorld,
    offset: Translation<f32>,
    current_drag: Option<Drag>,
    selected: Option<SelectedObject>,
    // TODO turn this into a ncollide world
    hitboxes: HashMap<HitboxKey, Hitbox>,
    connection: Connection<MessageToServer, MessageToClient>,
    drawable_window_size: (f32, f32),
    player_id: PlayerId,
    quitting: bool,
    crash: Option<String>,
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
            world,
            offset,
            current_drag: None,
            selected: None,
            hitboxes,
            connection,
            drawable_window_size: (0.0, 0.0),
            player_id,
            quitting: false,
            crash: None,
        };
        Ok(s)
    }

    fn draw_tile(&self, ctx: &mut Context, tile: &Tile) {
        let sprite_index = match tile.tile_type {
            TileType::Plains => SPRITE_TILE_PLAINS,
            TileType::Ocean => SPRITE_TILE_OCEAN,
            TileType::Mountain => SPRITE_TILE_MOUNTAIN,
            TileType::Desert => SPRITE_TILE_DESERT,
        };

        self.draw_tile_sprite(ctx, tile.position, sprite_index, None);

        if let Some(resource) = tile.resource {
            use ResourceType::*;
            let sprite_index = match resource {
                Sheep => SPRITE_RESOURCE_SHEEP,
                Horses => SPRITE_RESOURCE_HORSES,
                Gold => SPRITE_RESOURCE_GOLD,
                Iron => SPRITE_RESOURCE_IRON,
                Silver => SPRITE_RESOURCE_SILVER,
                Niter => SPRITE_RESOURCE_NITER,
                Coal => SPRITE_RESOURCE_COAL,
                Wheat => SPRITE_RESOURCE_WHEAT,
            };

            self.draw_tile_sprite(ctx, tile.position, sprite_index, None);
        }

        if let Some(vegetation) = tile.vegetation {
            use Vegetation::*;
            let sprite_index = match vegetation {
                Forest => SPRITE_FOREST,
                Jungle => SPRITE_JUNGLE,
            };
            self.draw_tile_sprite(ctx, tile.position, sprite_index, None);
        }
    }

    fn in_drawable_bounds(&self, dest_point: mint::Point2<f32>, width: f32, height: f32) -> bool {
        dest_point.x < self.drawable_window_size.0 + width &&
        dest_point.y < self.drawable_window_size.1 + height &&
        dest_point.x > -width &&
        dest_point.y > -height
    }

    fn draw(&self, ctx: &mut Context, pos: TilePosition, sprite_index: usize, color: Option<graphics::Color>, rotation: f32) {
        let dest_point = self.offset * get_tile_window_pos(pos);
        let mut dest_point = mint::Point2 { x: dest_point.x, y: dest_point.y };

        if !self.in_drawable_bounds(dest_point, TILE_WIDTH, TILE_HEIGHT) {
            return;
        }

        let offset = mint::Point2 { x: 0.5, y: 0.5 };
        dest_point.x += TILE_WIDTH * offset.x;
        dest_point.y += TILE_HEIGHT * offset.y;

        let mut params = DrawParam::default()
            .src(get_tile_image_src_rect(sprite_index))
            .dest(dest_point)
            .offset(offset)
            .rotation(rotation);

        if let Some(color) = color {
            params = params.color(color);
        }

        graphics::draw(ctx, &self.tile_sprites, params).unwrap();
    }

    fn draw_yields(&self, ctx: &mut Context, pos: TilePosition, yields: Yields) {
        let dest_center = self.offset * get_tile_window_pos(pos);
        let dest_center = mint::Point2 {
            x: dest_center.x + TILE_WIDTH * 0.5,
            y: dest_center.y + TILE_HEIGHT * 0.70,
        };

        if !self.in_drawable_bounds(dest_center, TILE_WIDTH, TILE_HEIGHT) {
            return;
        }

        let intra_padding: f32 = 2.0;
        let inter_padding: f32 = 5.0;

        let yields_width_for_type = |num| {
            match num {
                0 => 0.0,
                1 | 2 => YIELD_ICON_WIDTH,
                3 | 4 => YIELD_ICON_WIDTH * 2.0 + intra_padding,
                _ => panic!("yield too big to render"),
            }
        };

        let mut total_width = 0.0;
        let mut yield_calc = vec![];
        let yield_types = &[
            (SPRITE_YIELD_FOOD, yields.food),
            (SPRITE_YIELD_PRODUCTION, yields.production),
            (SPRITE_YIELD_SCIENCE, yields.science),
        ];
        for &(sprite_index, yield_value) in yield_types {
            if yield_value > 0 {
                let width = yields_width_for_type(yield_value);
                yield_calc.push((sprite_index, yield_value, total_width, width));
                total_width += width + inter_padding;
            }
        }

        for (sprite_index, num, x_start_offset, yield_width) in yield_calc {
            let src = get_yield_image_src_rect(sprite_index);

            for i in 0..num {
                let offset = mint::Point2 { x: 0.5, y: 0.5 };

                let extra_y = match (num, i) {
                    (1, 0) => 0.0,
                    (2, 0) | (3, 0) | (4, 0) | (4, 1)=> -(YIELD_ICON_HEIGHT + intra_padding) / 2.0,
                    (2, 1) | (3, 1) | (3, 2) | (4, 2) | (4, 3) => (YIELD_ICON_HEIGHT + intra_padding) / 2.0,
                    _ => unreachable!(),
                };

                let extra_x = match (num, i) {
                    (1, 0) => 0.0,
                    (2, _) => 0.0,
                    (3, 0) => yield_width / 4.0,
                    _ => (i as f32 % 2.0) * (YIELD_ICON_WIDTH + intra_padding),
                };

                let dest = mint::Point2 {
                    x: dest_center.x - total_width / 2.0 + YIELD_ICON_WIDTH * offset.x + x_start_offset + extra_x,
                    y: dest_center.y + YIELD_ICON_WIDTH * offset.y + extra_y,
                };

                let params = DrawParam::default().dest(dest).src(src).offset(offset);

                graphics::draw(ctx, &self.yield_sprites, params).unwrap();
            }
        }
    }

    fn draw_river(&self, ctx: &mut Context, pos: TilePosition, side: TileEdge) {
        self.draw(ctx, pos, SPRITE_RIVER, None, std::f32::consts::PI * 2.0 * (side.index() as f32) / 6.0);
    }

    fn draw_border(&self, ctx: &mut Context, pos: TilePosition, side: TileEdge, id: CivilizationId) {
        let [x, y, z] = self.world.civilization(id).unwrap().color().percents();
        let color = graphics::Color::new(x, y, z, 1.0);
        self.draw(ctx, pos, SPRITE_BORDER, Some(color), std::f32::consts::PI * 2.0 * (side.index() as f32) / 6.0);
    }

    fn draw_tile_sprite(&self, ctx: &mut Context, pos: TilePosition, sprite_index: usize, color: Option<graphics::Color>) {
        self.draw(ctx, pos, sprite_index, color, 0.0);
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

impl ggez_goodies::scene::Scene<SharedData, InputEvent> for InGameState {
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

        graphics::clear(ctx, graphics::BLACK);

        // Render game stuff
        {
            for tile in self.world.map.tiles() {
                self.draw_tile(ctx, tile);
            }

            for tile in self.world.map.tiles() {
                for river in &tile.rivers {
                    self.draw_river(ctx, tile.position, *river);
                }
            }

            for city in self.world.cities() {
                for border in city.borders() {
                    self.draw_border(ctx, border.0, border.1, city.owner())
                }
            }

            for unit in self.world.units() {
                let sprite_index = match unit.unit_type() {
                    UnitType::Civilian => SPRITE_CIVILIAN,
                    UnitType::Soldier => SPRITE_SOLDIER,
                };
                let [r, g, b] = self.world.civilization(unit.owner()).unwrap().color().percents();
                let color = if unit.remaining_movement() > 0 {
                    Some(graphics::Color::new(r, g, b, 1.0))
                } else {
                    Some(graphics::Color::new(r * 0.7, g *  0.7, b * 0.7, 0.95))
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

                        if self.can_control_unit(unit) && ggez::input::mouse::button_pressed(ctx, MouseButton::Right) {
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

            for tile in self.world.map.tiles() {
                self.draw_yields(ctx, tile.position, tile.yields());
            }
        }

        // Render game ui
        {
            use imgui::*;

            let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);

            let Self { player_id: you_player_id, world, selected, ref offset, connection, quitting, .. } = self;

            let you_civ_id = world.player(*you_player_id).unwrap().civilization_id();

            let fps = ggez::timer::fps(ctx);

            //  TODO use ui.current_font_size() to size buttons, etc
            let func = |ui: &imgui::Ui, fonts: &ImGuiFonts| {
                let window_padding = ui.clone_style().window_padding;

                const LEFT_WINDOW_WIDTH: f32 = 250.0;
                imgui::Window::new(im_str!("General"))
                    .size([LEFT_WINDOW_WIDTH, screen_height], imgui::Condition::Always)
                    .position([0.0, screen_height - screen_height], imgui::Condition::Always)
                    .collapsible(false)
                    .movable(false)
                    .resizable(false)
                    .build(ui, || {
                        let button_size = [LEFT_WINDOW_WIDTH - window_padding[0] * 2.0, 40.0];
                        if ui.button(im_str!("Quit"), button_size) {
                            *quitting = true;
                        }

                        ui.spacing();
                        ui.separator();
                        ui.text(format!("FPS: {:.0}", fps));
                        ui.text(format!("World: {}x{}", world.map.width(), world.map.height()));
                        ui.text(if cfg!(debug_assertions) { "Debug mode" } else { "Release mode" });
                        ui.spacing();
                        ui.separator();
                        ui.spacing();
                        ui.text("Players:");
                        for player in world.players() {
                            let you_str = if player.id() == *you_player_id { " (you)" } else { "" };
                            let ready_str = if player.ready() { " (ready)" } else { "" };
                            ui.text(format!("{}{}{}", player.name(), you_str, ready_str));
                        }
                        ui.spacing();
                        ui.separator();
                        ui.spacing();
                        ui.spacing();
                        ui.spacing();
                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        ui.text("Tasks:");
                        let mut todo_something = false;
                        for city in world.cities().filter(|city| city.owner() == you_civ_id) {
                            if city.producing().is_none() {
                                todo_something = true;
                                let clicked = ui.button(&ImString::new(format!("Set prod in {}", city.name())), button_size);
                                if clicked {
                                    *selected = Some(SelectedObject::City(city.id(), ImString::new(city.name())));
                                }
                            }
                        }

                        for unit in world.units().filter(|unit| unit.owner() == you_civ_id) {
                            if !unit.sleeping() && unit.remaining_movement() > 0 {
                                todo_something = true;
                                let clicked = ui.button(&ImString::new(format!("Move {} {}", unit.name(), unit.position())), button_size);
                                if clicked {
                                    *selected = Some(SelectedObject::Unit(unit.id()));
                                }
                            }
                        }

                        if !todo_something {
                            ui.text("Nothing");
                        }

                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        ui.text(format!("Turn {}", world.turn()));
                        let open_sans_semi_bold_30_handle = ui.push_font(fonts.open_sans_semi_bold_30);
                        let you_ready = world.player(*you_player_id).unwrap().ready();
                        let turn_button_label = if you_ready { im_str!("Waiting for players") } else { im_str!("Next turn") };
                        let next_turn_clicked = ui.button(turn_button_label, button_size);
                        open_sans_semi_bold_30_handle.pop(ui);
                        if next_turn_clicked {
                            connection.send_message(MessageToServer::Action(GameActionType::SetReady(!you_ready)));
                        }
                    });

                if let Some(selected) = selected {
                    const SIDEBAR_WIDTH: f32 = 350.0;
                    let sidebar_button_size: [f32; 2] = [SIDEBAR_WIDTH - window_padding[0] * 2.0, 40.0];
                    imgui::Window::new(im_str!("Selection"))
                        .size([SIDEBAR_WIDTH, screen_height], imgui::Condition::Always)
                        .position([screen_width - SIDEBAR_WIDTH, 0.0], imgui::Condition::Always)
                        .collapsible(false)
                        .movable(false)
                        .resizable(false)
                        .build(ui, || {
                            match selected {
                                SelectedObject::Tile(pos) => {
                                    let tile = world.map.tile(*pos);
                                    let tile_type = tile.tile_type;
                                    ui.text(format!("{} tile at {}", tile_type, pos));

                                    let mut things = vec![];
                                    if let Some(resource) = tile.resource {
                                        things.push(format!("{}", resource));
                                    }
                                    if let Some(vegetation) = tile.vegetation {
                                        things.push(format!("{}", vegetation));
                                    }
                                    if things.len() > 0 {
                                        ui.text(things.join(", "));
                                    }

                                    ui.spacing();

                                    let yields = tile.yields();
                                    if yields.food > 0 {
                                        ui.text(format!("{} food", yields.food));
                                    }
                                    if yields.production > 0 {
                                        ui.text(format!("{} production", yields.production));
                                    }
                                    if yields.science > 0 {
                                        ui.text(format!("{} science", yields.science));
                                    }
                                },
                                SelectedObject::Unit(unit_id) => {
                                    let unit = world.unit(*unit_id).unwrap();
                                    let owner_name = world.civilization(unit.owner()).unwrap().player_name();
                                    ui.text(format!("{} at {}", unit.name(), unit.position()));
                                    ui.text(format!("Type: {}", unit.unit_type()));
                                    ui.text(format!("Owner: {}", owner_name));
                                    ui.text(format!("Movement: {}/{}", unit.remaining_movement(), unit.total_movement()));
                                    ui.spacing();
                                    ui.spacing();
                                    ui.separator();
                                    ui.spacing();
                                    ui.spacing();

                                    let mut sleeping = unit.sleeping();
                                    if ui.checkbox(im_str!("Sleeping"), &mut sleeping) {
                                        connection.send_message(MessageToServer::Action(GameActionType::SetSleeping { unit_id: *unit_id, sleeping }));
                                    }

                                    ui.spacing();

                                    if unit.has_settle_ability() {
                                        // TODO disable button when can't settle
                                        let founding_city = ui.button(im_str!("Found city"), sidebar_button_size);
                                        if founding_city {
                                            let action = GameActionType::FoundCity { unit_id: *unit_id };
                                            connection.send_message(MessageToServer::Action(action));
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
                                        connection.send_message(MessageToServer::Action(action));
                                    }

                                    ui.spacing();
                                    ui.separator();
                                    ui.spacing();
                                    if let Some((producing_unit, producing_progress)) = city.producing() {
                                        ui.text(format!("Production: {}", producing_unit.name));
                                        let production_remaining = producing_unit.production_cost - producing_progress;
                                        ui.text(format!(
                                            "{}/{}, {:.0} turns remaining",
                                            producing_progress,
                                            producing_unit.production_cost,
                                            (production_remaining as f32 / city.production() as f32).ceil(),
                                        ));
                                    } else {
                                        ui.text("Production: None");
                                    }
                                    ui.spacing();
                                    ui.separator();
                                    ui.spacing();
                                    ui.text(im_str!("Production List"));
                                    for unit_template in world.unit_template_manager().all() {
                                        let label = format!("{}: {}", unit_template.name, unit_template.production_cost);
                                        let chose = ui.button(&ImString::new(label), sidebar_button_size);
                                        if chose {
                                            let action = GameActionType::SetProducing { city_id: *city_id, producing: Some(unit_template.clone()) };
                                            connection.send_message(MessageToServer::Action(action));
                                        }
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
            shared_data.imgui_wrapper.render(ctx, shared_data.hidpi_factor, func);
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
                    let (dx, dy) = drag.get_map_offset_delta(x, y);

                    self.offset.x += dx;
                    self.offset.y += dy;
                }
            }
            InputEvent::MouseDownEvent { button, x, y } => {
                if let MouseButton::Left = button {
                    self.current_drag = Some(Drag::new(x, y));
                }
            }
            InputEvent::MouseUpEvent { button, x, y } => {
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
            InputEvent::ScrollEvent { x: _x, y: _y } => {

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
