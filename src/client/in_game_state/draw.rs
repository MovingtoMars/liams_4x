use ggez::Context;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::graphics::Rect;
use ggez::mint;
use imgui::ImString;
use ncollide2d::math::Translation;

use crate::common::{
    GameActionType,
    TilePosition,
    MessageToServer,
    Tile,
    TileType,
    UnitType,
    CivilizationId,
    TileEdge,
    ResourceType,
    Yields,
    Vegetation,
    Citizen,
    Unit,
    UnitAbility,
};

use super::InGameState;
use super::super::constants::*;
use super::super::imgui_wrapper::ImGuiRenderContext;
use super::super::selected_object::SelectedObject;
use super::super::utils::get_tile_window_pos;

const CENTER_OFFSET: mint::Point2<f32> = mint::Point2 { x: 0.5, y: 0.5 };

fn get_tile_image_src_rect(index: usize) -> Rect {
    get_image_src_rect(index, 10, 8)
}

fn get_yield_image_src_rect(index: usize) -> Rect {
    get_image_src_rect(index, 5, 5)
}

fn get_citizen_image_src_rect(index: usize) -> Rect {
    get_image_src_rect(index, 3, 3)
}

fn get_image_src_rect(index: usize, x_count: usize, y_count: usize) -> Rect {
    let x = (index as f32) / x_count as f32 % 1.0;
    let y = ((index as f32) / x_count as f32).floor() / y_count as f32;

    Rect::new(x, y, 1.0 / x_count as f32, 1.0 / y_count as f32)
}

impl InGameState {
    pub(super) fn draw_tile(&self, ctx: &mut Context, tile: &Tile) {
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

    pub(super) fn in_drawable_bounds(&self, dest_point: mint::Point2<f32>, width: f32, height: f32) -> bool {
        dest_point.x < self.drawable_window_size.0 + width &&
        dest_point.y < self.drawable_window_size.1 + height &&
        dest_point.x > -width &&
        dest_point.y > -height
    }

    pub(super) fn draw(&self, ctx: &mut Context, pos: TilePosition, sprite_index: usize, color: Option<graphics::Color>, rotation: f32) {
        let dest_point = self.offset * get_tile_window_pos(pos);
        let mut dest_point = mint::Point2 { x: dest_point.x * self.zoom, y: dest_point.y * self.zoom };

        if !self.in_drawable_bounds(dest_point, TILE_WIDTH * self.zoom, TILE_HEIGHT * self.zoom) {
            return;
        }

        let offset = CENTER_OFFSET;
        let zoom = mint::Point2 { x: self.zoom, y: self.zoom };
        dest_point.x += TILE_WIDTH * offset.x * self.zoom;
        dest_point.y += TILE_HEIGHT * offset.y * self.zoom;

        let mut params = DrawParam::default()
            .src(get_tile_image_src_rect(sprite_index))
            .dest(dest_point)
            .offset(offset)
            .rotation(rotation)
            .scale(zoom);

        if let Some(color) = color {
            params = params.color(color);
        }

        graphics::draw(ctx, &self.tile_sprites, params).unwrap();
    }

    pub(super) fn draw_citizen_sprite(&self, ctx: &mut Context, pos: TilePosition, citizen: Option<Citizen>) {
        let dest_center = self.offset * get_tile_window_pos(pos);
        // TODO put this in a function
        let dest_center = mint::Point2 {
            x: (dest_center.x + TILE_WIDTH * 0.5) * self.zoom,
            y: (dest_center.y + TILE_HEIGHT * 0.5) * self.zoom,
        };

        if !self.in_drawable_bounds(dest_center, TILE_WIDTH * self.zoom, TILE_HEIGHT * self.zoom) {
            return;
        }

        let sprite_index = match citizen {
            None => SPRITE_CITIZEN_NONE,
            Some(Citizen::Normal) => SPRITE_CITIZEN_NORMAL,
            Some(Citizen::Locked) => SPRITE_CITIZEN_LOCKED,
        };
        let src = get_citizen_image_src_rect(sprite_index);

        let zoom = mint::Point2 { x: self.zoom, y: self.zoom };
        let params = DrawParam::default().dest(dest_center).src(src).offset(CENTER_OFFSET).scale(zoom);

        graphics::draw(ctx, &self.citizen_sprites, params).unwrap();
    }

    pub(super) fn draw_yields(&self, ctx: &mut Context, pos: TilePosition, yields: Yields) {
        let dest_center = self.offset * get_tile_window_pos(pos);
        let dest_center = mint::Point2 {
            x: (dest_center.x + TILE_WIDTH * 0.5) * self.zoom,
            y: (dest_center.y + TILE_HEIGHT * 0.70) * self.zoom,
        };

        if !self.in_drawable_bounds(dest_center, TILE_WIDTH * self.zoom, TILE_HEIGHT * self.zoom) {
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
            let yield_value = yield_value as usize;
            if yield_value > 0 {
                let width = yields_width_for_type(yield_value);
                yield_calc.push((sprite_index, yield_value, total_width, width));
                total_width += width + inter_padding;
            }
        }

        for (sprite_index, num, x_start_offset, yield_width) in yield_calc {
            let src = get_yield_image_src_rect(sprite_index);

            for i in 0..num {
                let offset = CENTER_OFFSET;

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
                    x: dest_center.x - (total_width / 2.0 - YIELD_ICON_WIDTH * offset.x - x_start_offset - extra_x) * self.zoom,
                    y: dest_center.y + (YIELD_ICON_WIDTH * offset.y + extra_y) * self.zoom,
                };

                let zoom = mint::Point2 { x: self.zoom, y: self.zoom };
                let params = DrawParam::default().dest(dest).src(src).offset(CENTER_OFFSET).scale(zoom);

                graphics::draw(ctx, &self.yield_sprites, params).unwrap();
            }
        }
    }

    pub(super) fn draw_river(&self, ctx: &mut Context, pos: TilePosition, side: TileEdge) {
        self.draw(ctx, pos, SPRITE_RIVER, None, std::f32::consts::PI * 2.0 * (side.index() as f32) / 6.0);
    }

    pub(super) fn draw_rivers(&self, ctx: &mut Context) {
        for tile in self.world.map.tiles() {
            for river in &tile.rivers {
                self.draw_river(ctx, tile.position, *river);
            }
        }
    }

    pub(super) fn draw_cities_borders(&self, ctx: &mut Context) {
        for city in self.world.cities() {
            for border in city.borders() {
                self.draw_border(ctx, border.0, border.1, city.owner())
            }
        }
    }

    pub(super) fn draw_tiles(&self, ctx: &mut Context) {
        for tile in self.world.map.tiles() {
            self.draw_tile(ctx, tile);
        }
    }

    pub(super) fn draw_tiles_yields(&self, ctx: &mut Context) {
        for tile in self.world.map.tiles() {
            self.draw_yields(ctx, tile.position, tile.yields());
        }
    }

    pub(super) fn draw_border(&self, ctx: &mut Context, pos: TilePosition, side: TileEdge, id: CivilizationId) {
        let [x, y, z] = self.world.civilization(id).unwrap().color().percents();
        let color = graphics::Color::new(x, y, z, 1.0);
        self.draw(ctx, pos, SPRITE_BORDER, Some(color), std::f32::consts::PI * 2.0 * (side.index() as f32) / 6.0);
    }

    pub(super) fn draw_tile_sprite(&self, ctx: &mut Context, pos: TilePosition, sprite_index: usize, color: Option<graphics::Color>) {
        self.draw(ctx, pos, sprite_index, color, 0.0);
    }

    fn draw_unit(&self, ctx: &mut Context, unit: &Unit) {
        let sprite_index = match unit.unit_type() {
            UnitType::Civilian => {
                if unit.has_ability(UnitAbility::Settle) {
                    SPRITE_SETTLER
                } else {
                    SPRITE_WORKER
                }
            },
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

    pub(super) fn draw_units(&self, ctx: &mut Context) {
        for unit in self.world.units() {
            self.draw_unit(ctx, unit);
        }
    }

    pub(super) fn draw_selected_highlight(&mut self, ctx: &mut Context) {
        if let Some(ref selected) = self.selected {
            match selected {
                SelectedObject::Tile(pos) => {
                    self.draw_tile_sprite(ctx, *pos, SPRITE_TILE_HIGHLIGHT, None);
                }
                SelectedObject::City(city_id, _) => {
                    let city = self.world.city(*city_id).unwrap();
                    let pos = city.position();
                    self.draw_tile_sprite(ctx, pos, SPRITE_TILE_HIGHLIGHT, None);

                    for (pos, citizen) in city.territory() {
                        if *pos != city.position() {
                            self.draw_citizen_sprite(ctx, *pos, *citizen);
                        }
                    }
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
    }

    pub(super) fn draw_cities_ui(&mut self, _ctx: &mut Context, rc: &ImGuiRenderContext) {
        let open_sans_semi_bold_30_handle = rc.ui.push_font(rc.fonts.open_sans_semi_bold_30);
        for city in self.world.cities() {
            let dest_point = self.offset * (Translation::new(0.0, 0.0) * get_tile_window_pos(city.position()));

            let width = TILE_INNER_WIDTH * 1.1;

            let selected = &mut self.selected;

            imgui::Window::new(&ImString::new(format!("city for tile {}", city.position())))
                .no_decoration()
                // .size([50.0, 30.0], imgui::Condition::Always)
                .position([dest_point.x * self.zoom  + (TILE_WIDTH * self.zoom - width) / 2.0 - 5.0, dest_point.y * self.zoom], imgui::Condition::Always)
                .always_auto_resize(true)
                .draw_background(false)
                .build(&rc.ui, || {
                    let clicked = rc.ui.button(&ImString::new(format!("{} {}", city.population(), city.name())), [width, 40.0]);
                    if clicked {
                        *selected = Some(SelectedObject::City(city.id(), ImString::new(city.name())));
                    }
                });
        }
        open_sans_semi_bold_30_handle.pop(&rc.ui);
    }

    pub(super) fn draw_general_sidebar_ui(&mut self, ctx: &mut Context, rc: &ImGuiRenderContext) {
        use imgui::*;

        let Rect { w: _screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);
        let window_padding = rc.ui.clone_style().window_padding;
        let fps = ggez::timer::fps(ctx);
        let you_civ_id = self.world.player(self.player_id).unwrap().civilization_id();

        const LEFT_WINDOW_WIDTH: f32 = 250.0;
        imgui::Window::new(im_str!("General"))
            .size([LEFT_WINDOW_WIDTH, screen_height], imgui::Condition::Always)
            .position([0.0, screen_height - screen_height], imgui::Condition::Always)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .build(&rc.ui, || {
                let button_size = [LEFT_WINDOW_WIDTH - window_padding[0] * 2.0, 40.0];
                if rc.ui.button(im_str!("Quit"), button_size) {
                    self.quitting = true;
                }

                rc.ui.spacing();
                rc.ui.separator();
                rc.ui.text(format!("FPS: {:.0}", fps));
                rc.ui.text(format!("World: {}x{}", self.world.map.width(), self.world.map.height()));
                rc.ui.text(if cfg!(debug_assertions) { "Debug mode" } else { "Release mode" });
                rc.ui.spacing();
                rc.ui.separator();
                rc.ui.spacing();
                rc.ui.text("Players:");
                for player in self.world.players() {
                    let you_str = if player.id() == self.player_id { " (you)" } else { "" };
                    let ready_str = if player.ready() { " (ready)" } else { "" };
                    rc.ui.text(format!("{}{}{}", player.name(), you_str, ready_str));
                }
                rc.ui.spacing();
                rc.ui.separator();
                rc.ui.spacing();
                rc.ui.spacing();
                rc.ui.spacing();
                rc.ui.spacing();
                rc.ui.separator();
                rc.ui.spacing();

                rc.ui.text("Tasks:");
                let mut todo_something = false;
                for city in self.world.cities().filter(|city| city.owner() == you_civ_id) {
                    if city.producing().is_none() {
                        todo_something = true;
                        let clicked = rc.ui.button(&ImString::new(format!("Set prod in {}", city.name())), button_size);
                        if clicked {
                            self.selected = Some(SelectedObject::City(city.id(), ImString::new(city.name())));
                        }
                    }
                }

                for unit in self.world.units().filter(|unit| unit.owner() == you_civ_id) {
                    if !unit.sleeping() && unit.remaining_movement() > 0 {
                        todo_something = true;
                        let clicked = rc.ui.button(&ImString::new(format!("Move {} {}", unit.name(), unit.position())), button_size);
                        if clicked {
                            self.selected = Some(SelectedObject::Unit(unit.id()));
                        }
                    }
                }

                if !todo_something {
                    rc.ui.text("Nothing");
                }

                rc.ui.spacing();
                rc.ui.separator();
                rc.ui.spacing();

                rc.ui.text(format!("Turn {}", self.world.turn()));
                let open_sans_semi_bold_30_handle = rc.ui.push_font(rc.fonts.open_sans_semi_bold_30);
                let you_ready = self.world.player(self.player_id).unwrap().ready();
                let turn_button_label = if you_ready { im_str!("Waiting for players") } else { im_str!("Next turn") };
                let next_turn_clicked = rc.ui.button(turn_button_label, button_size);
                open_sans_semi_bold_30_handle.pop(&rc.ui);
                if next_turn_clicked {
                    self.connection.send_message(MessageToServer::Action(GameActionType::SetReady(!you_ready)));
                }
            });
    }


    pub(super) fn draw_selected_sidebar_ui(&mut self, ctx: &mut Context, rc: &ImGuiRenderContext) {
        if self.selected.is_none() {
            return;
        }

        use imgui::*;

        let Rect { w: screen_width, h: screen_height, .. } = graphics::screen_coordinates(ctx);
        let window_padding = rc.ui.clone_style().window_padding;

        const SIDEBAR_WIDTH: f32 = 350.0;
        let sidebar_button_size: [f32; 2] = [SIDEBAR_WIDTH - window_padding[0] * 2.0, 40.0];
        imgui::Window::new(im_str!("Selection"))
            .size([SIDEBAR_WIDTH, screen_height], imgui::Condition::Always)
            .position([screen_width - SIDEBAR_WIDTH, 0.0], imgui::Condition::Always)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .build(&rc.ui, || {
                match self.selected.as_mut().unwrap() {
                    SelectedObject::Tile(pos) => {
                        let tile = self.world.map.tile(*pos);
                        let tile_type = tile.tile_type;
                        rc.ui.text(format!("{} tile at {}", tile_type, pos));

                        let mut things = vec![];
                        if let Some(resource) = tile.resource {
                            things.push(format!("{}", resource));
                        }
                        if let Some(vegetation) = tile.vegetation {
                            things.push(format!("{}", vegetation));
                        }
                        if things.len() > 0 {
                            rc.ui.text(things.join(", "));
                        }

                        rc.ui.spacing();

                        let yields = tile.yields();
                        if yields.food > 0.0 {
                            rc.ui.text(format!("{} food", yields.food));
                        }
                        if yields.production > 0.0 {
                            rc.ui.text(format!("{} production", yields.production));
                        }
                        if yields.science > 0.0 {
                            rc.ui.text(format!("{} science", yields.science));
                        }
                    },
                    SelectedObject::Unit(unit_id) => {
                        let unit = self.world.unit(*unit_id).unwrap();
                        let owner_name = self.world.civilization(unit.owner()).unwrap().player_name();
                        rc.ui.text(format!("{} at {}", unit.name(), unit.position()));
                        rc.ui.text(format!("Type: {}", unit.unit_type()));
                        rc.ui.text(format!("Owner: {}", owner_name));
                        rc.ui.text(format!("Movement: {}/{}", unit.remaining_movement(), unit.total_movement()));
                        rc.ui.spacing();
                        rc.ui.spacing();
                        rc.ui.separator();
                        rc.ui.spacing();
                        rc.ui.spacing();

                        let mut sleeping = unit.sleeping();
                        if rc.ui.checkbox(im_str!("Sleeping"), &mut sleeping) {
                            self.connection.send_message(MessageToServer::Action(GameActionType::SetSleeping { unit_id: *unit_id, sleeping }));
                        }

                        rc.ui.spacing();

                        if unit.has_ability(UnitAbility::Settle) {
                            // TODO disable button when can't settle
                            let founding_city = rc.ui.button(im_str!("Found city"), sidebar_button_size);
                            if founding_city {
                                let action = GameActionType::FoundCity { unit_id: *unit_id };
                                self.connection.send_message(MessageToServer::Action(action));
                            }
                        }
                    }
                    SelectedObject::City(city_id, ref mut city_name_buf) => {
                        let city = self.world.city(*city_id).unwrap();
                        // TODO name length limit?
                        let city_name_changed = rc.ui.input_text(im_str!(""), city_name_buf)
                            .resize_buffer(true)
                            // If the user holds down a key, it can send quite a lot of data.
                            // Perhaps debounce, or set this to false.
                            .enter_returns_true(false)
                            .build();

                        let owner_name = self.world.civilization(city.owner()).unwrap().player_name();
                        rc.ui.text(format!("Owner: {}", owner_name));
                        rc.ui.text(format!("City at {}", city.position()));

                        if city_name_changed {
                            let action = GameActionType::RenameCity { city_id: *city_id, name: city_name_buf.to_string() };
                            self.connection.send_message(MessageToServer::Action(action));
                        }

                        rc.ui.spacing();
                        rc.ui.separator();
                        rc.ui.spacing();

                        rc.ui.text(format!(
                            "Growth: {}/{:.2} ({} turns remaining)",
                            city.accumulated_food(),
                            city.required_food_for_population_increase(),
                            city.turns_until_population_increase(),
                        ));
                        if city.next_tile_to_expand_to(&self.world.map).is_some() {
                            rc.ui.text(format!(
                                "{} turns until territory expansion",
                                city.turns_until_territory_growth(),
                            ));
                        } else {
                            rc.ui.text(format!("No longer expanding territory"));
                        }
                        rc.ui.text(format!("Unemployed citizens: {}", city.unemployed_citizen_count()));

                        rc.ui.spacing();
                        rc.ui.separator();
                        rc.ui.spacing();

                        let yields = city.yields();

                        rc.ui.text(format!("Population: {}", city.population()));
                        rc.ui.text(format!("Food: {}", yields.food));
                        rc.ui.text(format!("Production: {}", yields.production));
                        rc.ui.text(format!("Science: {}", yields.science));

                        rc.ui.spacing();
                        rc.ui.separator();
                        rc.ui.spacing();
                        if let Some((producing_unit, producing_progress)) = city.producing() {
                            rc.ui.text(format!("Production: {}", producing_unit.name));
                            let production_remaining = producing_unit.production_cost - producing_progress;
                            rc.ui.text(format!(
                                "{}/{} ({} turns remaining)",
                                producing_progress,
                                producing_unit.production_cost,
                                (production_remaining as f32 / yields.production as f32).ceil() as usize,
                            ));
                        } else {
                            rc.ui.text("Production: None");
                        }
                        rc.ui.spacing();
                        rc.ui.separator();
                        rc.ui.spacing();

                        rc.ui.text(im_str!("Production List"));
                        for unit_template in self.world.unit_template_manager().all() {
                            let label = format!(
                                "{}: {} ({} turns)",
                                unit_template.name,
                                unit_template.production_cost,
                                unit_template.turn_cost(yields.production),
                            );

                            let chose = rc.ui.button(&ImString::new(label), sidebar_button_size);
                            if chose {
                                let action = GameActionType::SetProducing { city_id: *city_id, producing: Some(unit_template.clone()) };
                                self.connection.send_message(MessageToServer::Action(action));
                            }
                        }
                    }
                }
            });
    }
}
