use macroquad::color::Color;
use macroquad::input::{KeyCode, is_key_down};
use macroquad::math::Rect;
use macroquad::prelude::{
    BLACK, DrawTextureParams, TextParams, Texture2D, Vec2, Vec3, WHITE, draw_text_ex,
    draw_texture_ex,
};
use macroquad::rand::RandGenerator;
use macroquad::text::draw_text;
use macroquad::time::get_frame_time;
use probabilistic_models::traits::{ReadChoiceLabels, ReadStateSpace, ReadValuations};
use probabilistic_models::{ChoiceIndex, Index, RawIndex, StateIndex};

#[derive(Clone)]
pub struct Entity {
    position: Vec3,
    size: Vec2,
    texture: EntityVariants,
}

#[derive(Clone)]
pub enum EntityVariants {
    Player {
        direction: PlayerDirection,
        alt: bool,
    },
}

impl EntityVariants {
    pub fn rect_and_rotation(&self) -> (Rect, f32) {
        match self {
            EntityVariants::Player { direction, alt } => {
                let (mut x, y) = match direction {
                    PlayerDirection::Left => (5, 3),
                    PlayerDirection::Right => (3, 3),
                    PlayerDirection::Up => (7, 3),
                    PlayerDirection::Down => (9, 3),
                };
                if *alt {
                    x += 1
                };
                (
                    Rect {
                        x: x as f32 * 16.0,
                        y: y as f32 * 16.0,
                        w: 16.0,
                        h: 16.0,
                    },
                    0.0,
                )
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum PlayerDirection {
    Left,
    Right,
    Up,
    Down,
}

impl Entity {
    pub fn draw(&self, camera: &Camera, texture: &Texture2D) {
        let actual_position = Vec3::new(
            (self.position.x * 16.0).round() * (1.0 / 16.0),
            (self.position.y * 16.0).round() * (1.0 / 16.0),
            (self.position.z * 16.0).round() * (1.0 / 16.0),
        );
        let position = camera.project(actual_position);
        let size = camera.project_size(self.size);
        let (rect, rotation) = self.texture.rect_and_rotation();
        draw_texture_ex(
            texture,
            position.x - size.x * 0.5,
            position.y - size.y * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                source: Some(rect),
                rotation,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        )
    }
}

pub struct Player {
    position: Vec3,
    velocity: Vec3,
    width: f32,
    height: f32,
    last_direction: PlayerDirection,
    distance: f32,
}

impl Player {
    pub fn walk_cycle(&self) -> bool {
        self.distance % 0.8 < 0.4
    }

    pub fn update_direction(&mut self) -> PlayerDirection {
        if self.velocity.length() < 0.05 {
            self.last_direction
        } else {
            let direction = if self.velocity.x.abs() > self.velocity.z.abs() {
                if self.velocity.x < 0.0 {
                    PlayerDirection::Left
                } else {
                    PlayerDirection::Right
                }
            } else {
                if self.velocity.z < 0.0 {
                    PlayerDirection::Down
                } else {
                    PlayerDirection::Up
                }
            };
            self.last_direction = direction;
            direction
        }
    }
}

pub enum Block {
    #[allow(unused)]
    Air,
    SolidAir,
    Floor,
    Pillar {
        segment: u8,
    },
    Bridge {
        quality: u8,
        segment: u8,
    },
    StairLeft,
    StairRight,
    SlideLeft,
    SlideRight,
    Sign {
        left: bool,
        segment: u8,
        text: String,
    },
    Tube {
        segment: u8,
        text: String,
    },
    StoneLabel {
        text: String,
    },
}

impl Block {
    pub fn draw(&self, x: usize, y: usize, z: usize, camera: &Camera, texture: &Texture2D) {
        let y = y as isize;
        match self {
            Block::Air => {}
            Block::SolidAir => {}
            Block::Floor => {
                let noise1 = Self::noise(x, y, z) % 2;
                let noise2 = Self::noise(x + 2, y + 3, z) % 2;
                Self::draw_internal(x, y - 1, z, 0 + noise1, 0, camera, texture);
                Self::draw_internal(x, y, z, 0 + noise2, 1, camera, texture);
            }
            Block::Pillar { segment } => {
                Self::draw_internal(x, y - 1, z, 2, *segment as usize, camera, texture);
                if *segment < 5 {
                    Self::draw_internal(x, y, z, 2, *segment as usize + 1, camera, texture);
                }
            }
            Block::Bridge { quality, segment } => {
                if *quality == 4 {
                    if *segment == 0 {
                        Self::draw_internal(x, y - 1, z, 9, 0, camera, texture);
                    } else if *segment == 2 {
                        Self::draw_internal(x, y - 1, z, 10, 0, camera, texture);
                    }
                } else {
                    let (start_x, start_y) = match *quality {
                        0 => (3, 0),
                        1 => (6, 0),
                        2 => (3, 1),
                        3 => (6, 1),
                        _ => unreachable!(),
                    };
                    Self::draw_internal(
                        x,
                        y - 1,
                        z,
                        start_x + *segment as usize,
                        start_y,
                        camera,
                        texture,
                    );
                }
            }
            Block::StairLeft => {
                Self::draw_internal(x, y - 1, z, 12, 0, camera, texture);
                Self::draw_internal(x, y, z, 12, 1, camera, texture);
            }
            Block::StairRight => {
                Self::draw_internal(x, y - 1, z, 11, 0, camera, texture);
                Self::draw_internal(x, y, z, 11, 1, camera, texture);
            }
            Block::SlideLeft => {
                Self::draw_internal(x, y - 1, z, 14, 0, camera, texture);
                Self::draw_internal(x, y, z, 14, 1, camera, texture);
            }
            Block::SlideRight => {
                Self::draw_internal(x, y - 1, z, 13, 0, camera, texture);
                Self::draw_internal(x, y, z, 13, 1, camera, texture);
            }
            Block::Sign {
                left,
                segment,
                text,
            } => {
                let start = if *left { 5 } else { 3 };
                Self::draw_internal(x, y, z, start + *segment as usize, 2, camera, texture);
                if *segment == 1 {
                    let offset = if *left { 0.27 } else { 0.1 };
                    let pos = camera.project(Vec3::new(
                        x as f32 + offset - 1.0,
                        y as f32 + 0.42,
                        z as f32,
                    ));
                    let size = camera.project_size(Vec2::new(0.0, 0.57));
                    draw_text(text, pos.x, pos.y, size.y, BLACK);
                }
            }
            Block::Tube { segment, text } => {
                Self::draw_internal(x, y - 1, z, 0, 2 + *segment as usize, camera, texture);
                if *segment == 0 {
                    let pos = camera.project(Vec3::new(x as f32 + 0.4, y as f32, z as f32));
                    let size = camera.project_size(Vec2::new(0.0, 0.57));
                    draw_text_ex(
                        text,
                        pos.x,
                        pos.y,
                        TextParams {
                            font: None,
                            font_size: size.y as u16,
                            font_scale: 1.0,
                            font_scale_aspect: 1.0,
                            rotation: std::f32::consts::PI * 0.5,
                            color: BLACK,
                        },
                    );
                }
            }
            Block::StoneLabel { text } => {
                let pos = camera.project(Vec3::new(x as f32, y as f32 + 0.7, z as f32));
                let size = camera.project_size(Vec2::new(0.0, 0.57));
                draw_text(text, pos.x, pos.y, size.y, Color::new(0.8, 0.8, 0.8, 1.0));
            }
        }
    }

    pub fn intersects(&self, x: usize, y: usize, z: usize, other: &AABB) -> bool {
        match self {
            Block::Air | Block::SolidAir => false,
            Block::Floor | Block::Pillar { .. } => AABB::int_cube(x, y, z).intersects(other),
            Block::Bridge { segment, quality } => {
                if *quality == 4 {
                    false
                } else {
                    const SAG: f32 = 0.2;
                    if *segment == 0 || *segment == 2 {
                        let segs = 32;
                        let seg_size = 1.0 / segs as f32;
                        for i in 0..32 {
                            let height = if *segment == 0 {
                                (i as f32 * seg_size) * SAG
                            } else {
                                ((segs - i - 1) as f32 * seg_size) * SAG
                            };
                            let aabb = AABB::from_pos_and_size(
                                Vec3::new(
                                    x as f32 + i as f32 * seg_size,
                                    y as f32 + height,
                                    z as f32,
                                ),
                                Vec3::new(0.1, 1.0 - height, 1.0),
                            );
                            if aabb.intersects(other) {
                                return true;
                            }
                        }
                        false
                    } else {
                        let aabb = AABB::from_pos_and_size(
                            Vec3::new(x as f32, y as f32 + SAG, z as f32),
                            Vec3::new(1.0, 1.0 - SAG, 1.0),
                        );
                        aabb.intersects(other)
                    }
                }
            }
            Block::StairLeft | Block::StairRight | Block::SlideLeft | Block::SlideRight => {
                let segs = 32;
                let seg_size = 1.0 / segs as f32;
                for i in 0..32 {
                    let height =
                        if matches!(self, Block::StairLeft) || matches!(self, Block::SlideLeft) {
                            i as f32 * seg_size
                        } else {
                            (segs - i - 1) as f32 * seg_size
                        };
                    let aabb = AABB::from_pos_and_size(
                        Vec3::new(x as f32 + i as f32 * seg_size, y as f32 + height, z as f32),
                        Vec3::new(0.1, 1.0 - height, 1.0),
                    );
                    if aabb.intersects(other) {
                        return true;
                    }
                }
                false
            }
            Block::Sign { .. } => AABB::from_pos_and_size(
                Vec3::new(x as f32, y as f32, z as f32),
                Vec3::new(1.0, 1.0, 0.2),
            )
            .intersects(other),
            Block::Tube { .. } => false,
            Block::StoneLabel { .. } => false,
        }
    }

    fn noise(x: usize, y: isize, z: usize) -> usize {
        (x * 33 + y.abs() as usize * 7 + z) * (x + 1)
    }

    fn draw_internal(
        x: usize,
        y: isize,
        z: usize,
        tx: usize,
        ty: usize,
        camera: &Camera,
        texture: &Texture2D,
    ) {
        let position = camera.project_ints(x, y, z);
        let size = camera.project_size(Vec2::new(1.0, 1.0));
        let rect = Rect::new(16.0 * (tx as f32), 16.0 * (ty as f32), 16.0, 16.0);
        draw_texture_ex(
            texture,
            position.x,
            position.y,
            Color::new(1.0, 1.0, 1.0, 1.0),
            DrawTextureParams {
                dest_size: Some(size),
                source: Some(rect),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        )
    }
}

pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn int_cube(x: usize, y: usize, z: usize) -> AABB {
        Self {
            min: Vec3::new(x as f32, y as f32, z as f32),
            max: Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
        }
    }
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_pos_and_size(min: Vec3, size: Vec3) -> Self {
        Self {
            min,
            max: min + size,
        }
    }
    pub fn from_center_and_size(center: Vec3, size: Vec3) -> Self {
        Self {
            min: center - size * 0.5,
            max: center + size * 0.5,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        ((self.min.x >= other.min.x && self.min.x <= other.max.x)
            || (other.min.x >= self.min.x && other.min.x <= self.max.x))
            && ((self.min.y >= other.min.y && self.min.y <= other.max.y)
                || (other.min.y >= self.min.y && other.min.y <= self.max.y))
            && ((self.min.z >= other.min.z && self.min.z <= other.max.z)
                || (other.min.z >= self.min.z && other.min.z <= self.max.z))
    }

    pub fn contains(&self, point: Vec3) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
            && self.min.z <= point.z
            && point.z <= self.max.z
    }
}

pub struct Room {
    player: Player,
    blocks: Vec<Vec<Vec<Block>>>,
    entities_per_layer: Vec<Vec<Entity>>,
    width: usize,
    height: usize,
    depth: usize,
    collapsing_triggers: Vec<CollapseTrigger>,
    delayed_collapses: Vec<(usize, usize, usize, f32)>,
    any_collapse: bool,
    teleport_zones: Vec<(AABB, usize)>,
    pub teleport_to: Option<usize>,
}

impl Room {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        let mut blocks = Vec::new();
        for _j in 0..depth {
            let mut layer = Vec::new();
            for _k in 0..height {
                let mut row = Vec::new();
                for _i in 0..width {
                    row.push(Block::SolidAir);
                }
                layer.push(row);
            }
            blocks.push(layer);
        }
        Self {
            player: Player {
                position: Default::default(),
                velocity: Default::default(),
                width: 0.8,
                height: 0.8,
                last_direction: PlayerDirection::Down,
                distance: 0.0,
            },
            blocks,
            entities_per_layer: vec![Vec::new(); depth],
            width,
            depth,
            collapsing_triggers: vec![],
            height,
            delayed_collapses: vec![],
            any_collapse: false,
            teleport_zones: vec![],
            teleport_to: None,
        }
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[z][y][x]
    }
    fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < self.width && y < self.height && z < self.depth {
            self.blocks[z][y][x] = block;
        } else {
            println!("Tried to place block out of bounds")
        }
    }

    fn intersects(&self, bb: &AABB) -> bool {
        let from = (bb.min - Vec3::new(1.0, 1.0, 1.0).floor())
            .max(Vec3::new(0.0, 0.0, 0.0))
            .as_uvec3();
        let to = (bb.max + Vec3::new(1.0, 1.0, 1.0).ceil())
            .min(Vec3::new(
                self.width as f32,
                self.height as f32,
                self.depth as f32,
            ))
            .as_uvec3();
        for x in from.x..to.x {
            for y in from.y..to.y {
                for z in from.z..to.z {
                    if self
                        .get_block(x as usize, y as usize, z as usize)
                        .intersects(x as usize, y as usize, z as usize, bb)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn insert_entity(&mut self, position: Vec3, size: Vec2, texture: EntityVariants) {
        let layer = (position.z).clamp(0.0, self.entities_per_layer.len() as f32 - 0.05) as usize;
        self.entities_per_layer[layer].push(Entity {
            position,
            size,
            texture,
        })
    }

    pub fn draw(&mut self, texture: &Texture2D, camera: &Camera) {
        for layer in &mut self.entities_per_layer {
            layer.clear();
        }
        let variant = EntityVariants::Player {
            direction: self.player.update_direction(),
            alt: self.player.walk_cycle(),
        };
        self.insert_entity(
            self.player.position + Vec3::new(0.0, -0.5, 0.0), // self.player.height *
            Vec2::new(1.0, 1.0),
            variant,
        );
        for layer in &mut self.entities_per_layer {
            layer.sort_unstable_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
        }
        for z in (0..self.depth).rev() {
            let entities = &self.entities_per_layer[z];
            let mut entity_index = 0;
            for y in (0..self.height).rev() {
                while entity_index < entities.len() && entities[entity_index].position.y >= y as f32
                {
                    entities[entity_index].draw(camera, texture);
                    entity_index += 1;
                }
                for x in 0..self.width {
                    self.get_block(x, y, z).draw(x, y, z, camera, texture);
                }
            }
        }
    }

    pub fn place_player(&mut self, x: usize, y: usize, z: usize) {
        self.player.position = Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.3);
    }
    pub fn player_pos(&self) -> Vec3 {
        self.player.position
    }
    fn player_bb(&self) -> AABB {
        AABB::from_pos_and_size(
            self.player.position - Vec3::new(self.player.width * 0.5, self.player.height, 0.1),
            Vec3::new(self.player.width, self.player.height, 0.2),
        )
    }

    pub fn update(&mut self) {
        let elapsed = get_frame_time().min(0.05);
        let mut direction = Vec2::new(0.0, 0.);
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            direction.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            direction.x += 1.0;
        }
        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }
        if self.any_collapse {
            direction = Vec2::new(0.0, 0.0);
        }
        let speed = 3.0;
        let acceleration = 30.0;
        direction *= speed;
        if self.player.velocity.x < direction.x {
            self.player.velocity.x =
                (self.player.velocity.x + acceleration * elapsed).min(direction.x);
        }
        if self.player.velocity.x > direction.x {
            self.player.velocity.x =
                (self.player.velocity.x - acceleration * elapsed).max(direction.x);
        }
        if self.player.velocity.z < direction.y {
            self.player.velocity.z =
                (self.player.velocity.z + acceleration * elapsed).min(direction.y);
        }
        if self.player.velocity.z > direction.y {
            self.player.velocity.z =
                (self.player.velocity.z - acceleration * elapsed).max(direction.y);
        }

        let climb_budget = self.player.velocity.length() * 1.7 * elapsed;
        let (moved, climb_budget) = self.move_player(
            self.player.position + Vec3::new(self.player.velocity.x * elapsed, 0.0, 0.0),
            climb_budget,
        );
        if !moved {
            self.player.velocity.x = 0.0;
        }

        let (moved, _climb_budget) = self.move_player(
            self.player.position + Vec3::new(0.0, 0.0, self.player.velocity.z * elapsed),
            climb_budget,
        );
        if !moved {
            self.player.velocity.z = 0.0;
        }
        self.player.velocity.y += 9.0 * elapsed;
        self.player.position.y += self.player.velocity.y * elapsed;
        if self.intersects(&self.player_bb()) {
            self.player.position.y -= self.player.velocity.y * elapsed;
            self.player.velocity.y = 0.0;
        }
        self.player.distance +=
            Vec2::new(self.player.velocity.x, self.player.velocity.z).length() * elapsed;

        for collapse_trigger in &mut self.collapsing_triggers {
            if collapse_trigger.activated {
                continue;
            }
            if collapse_trigger.zone.contains(self.player.position) {
                collapse_trigger.activated = true;
                self.any_collapse = true;
                for &(x, y, z, delay) in &collapse_trigger.tiles {
                    self.delayed_collapses.push((x, y, z, delay));
                }
            }
        }

        for i in (0..self.delayed_collapses.len()).rev() {
            let (x, y, z, delay) = &mut self.delayed_collapses[i];
            if *delay >= 0.0 {
                *delay -= elapsed;
                if *delay <= 0.0 {
                    if let Block::Bridge { quality, .. } = &mut self.blocks[*z][*y][*x] {
                        *quality = 4;
                    }
                }
            }
        }

        if self.teleport_to.is_none() {
            for (zone, to) in &self.teleport_zones {
                if zone.contains(self.player.position) {
                    self.teleport_to = Some(*to);
                }
            }
        }
    }

    fn move_player(&mut self, to: Vec3, climb_budget: f32) -> (bool, f32) {
        // true if moved successfully, f32 stores remaining climb budget
        let old_pos = self.player.position;
        let steps = 0.01;
        let mut counter = 0;
        loop {
            self.player.position = to - Vec3::new(0.0, steps * counter as f32, 0.0);
            if !self.intersects(&self.player_bb()) {
                return (true, climb_budget - steps * counter as f32);
            }
            if steps * counter as f32 >= climb_budget {
                self.player.position = old_pos;
                return (false, climb_budget);
            }
            counter += 1;
        }
    }

    pub fn carve_platform(&mut self, height: usize, x1: usize, z1: usize, x2: usize, z2: usize) {
        for x in x1..x2 {
            for z in z1..z2 {
                self.set_block(x, height, z, Block::Floor);
                //self.set_block(x, height - 1, z, Block::Air);
            }
        }
    }
    pub fn build_stair(&mut self, x: usize, height: usize, z: usize, length: usize, left: bool) {
        let offset: isize = if left { -1 } else { 1 };
        for i in 0..length {
            let block = if left {
                Block::StairLeft
            } else {
                Block::StairRight
            };
            self.set_block(
                (x as isize + i as isize * offset) as usize,
                height - i,
                z,
                block,
            );
        }
    }
    pub fn build_slide(&mut self, x: usize, height: usize, z: usize, length: usize, left: bool) {
        let offset: isize = if left { -1 } else { 1 };
        for i in 0..length {
            let block = if left {
                Block::SlideLeft
            } else {
                Block::SlideRight
            };
            self.set_block(
                (x as isize + i as isize * offset) as usize,
                height - i,
                z,
                block,
            );
        }
    }

    pub fn build_bridge(
        &mut self,
        x: usize,
        height: usize,
        z: usize,
        length: usize,
        probability: f32,
        collapsing: bool,
    ) {
        if collapsing {
            let mut tiles = Vec::new();
            let middle = (length - 1) / 2;
            for i in 0..length {
                let dist = (i as isize - middle as isize).abs();
                let delay = 0.11 * dist as f32;
                tiles.push((x + i, height, z, delay))
            }
            self.collapsing_triggers.push(CollapseTrigger {
                activated: false,
                zone: AABB::from_pos_and_size(
                    Vec3::new(
                        x as f32 + length as f32 * 0.5,
                        height as f32 - 0.5,
                        z as f32 - 0.5,
                    ),
                    Vec3::new(0.3, 1.5, 1.5),
                ),
                tiles,
            })
        }
        for i in 0..length {
            let segment = if i == 0 {
                0
            } else if i == length - 1 {
                2
            } else {
                1
            };
            let quality = if probability < 0.08 {
                0
            } else if probability < 0.2 {
                1
            } else if probability < 0.5 {
                2
            } else {
                3
            };
            self.set_block(x + i, height, z, Block::Bridge { quality, segment });
        }
    }

    pub fn build_pillar(&mut self, x: usize, height: usize, z: usize) {
        for i in 0..5 {
            self.set_block(x, height + i, z, Block::Pillar { segment: i as u8 });
        }
    }
    pub fn build_pillar_head(&mut self, x: usize, height: usize, z: usize, length: usize) {
        for i in 0..5.min(length) {
            self.set_block(x, height + i, z, Block::Pillar { segment: i as u8 });
        }
    }
    pub fn build_tube(&mut self, x: usize, height: usize, z: usize, text: String) {
        for i in 0..5 {
            self.set_block(
                x,
                height + i,
                z,
                Block::Tube {
                    segment: i as u8,
                    text: text.clone(),
                },
            );
        }
    }
    pub fn build_sign(&mut self, x: usize, height: usize, z: usize, left: bool, text: String) {
        for i in 0..2 {
            self.set_block(
                x + i,
                height,
                z,
                Block::Sign {
                    left,
                    segment: i as u8,
                    text: text.clone(),
                },
            );
        }
    }
}

pub struct RoomGenerator {
    room: Room,
    rng: RandGenerator,
    transition_rng: RandGenerator,
    sides: Vec<PlatformSide>,
    edges: Vec<PlatformEdge>,
}

pub struct CollapseTrigger {
    activated: bool,
    zone: AABB,
    tiles: Vec<(usize, usize, usize, f32)>,
}

pub struct PlatformSide {
    x: usize,
    height: usize,
    z1: usize,
    z2: usize,
    left: bool,
    used: Vec<bool>,
}

pub struct PlatformEdge {
    z: usize,
    x1: usize,
    x2: usize,
    height: usize,
    top: bool,
}

impl RoomGenerator {
    pub fn generate<M: ReadStateSpace + ReadChoiceLabels<E = Option<String>> + ReadValuations>(
        mdp: M,
        state: StateIndex<M::Ix>,
    ) -> Room {
        let (width, depth) = (150, 150);
        let rng = macroquad::rand::RandGenerator::new();
        rng.srand(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs());
        // rng.srand(state.raw().as_usize() as u64);
        let transition_rng = macroquad::rand::RandGenerator::new();
        transition_rng.srand(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs());

        let mut genr = Self {
            room: Room::new(width, 30, depth),
            rng,
            transition_rng,
            sides: vec![],
            edges: vec![],
        };
        let label = mdp.state_valuation(state).to_string();
        genr.initial_platform(label);

        for _ in 0..genr.rng.gen_range(0, 2) {
            genr.extend_platform();
        }
        for choice in mdp.choices_of_state(state) {
            genr.build_choice(&mdp, choice);
        }
        genr.room.teleport_zones.push((
            AABB::new(
                Vec3::new(0.0, 30.0, 0.0),
                Vec3::new(width as f32, 40.0, depth as f32),
            ),
            state.raw().as_usize(),
        ));

        genr.room
    }

    fn initial_platform(&mut self, label: String) {
        let width = self.rng.gen_range(5, 9);
        let depth = self.rng.gen_range(5, 9);
        let height = self
            .rng
            .gen_range(self.room.height / 4, self.room.height * 3 / 4);

        let x = self.room.width / 2 - width / 2;
        let z = self.room.depth / 2 - depth / 2;
        self.room.carve_platform(height, x, z, x + width, z + depth);
        let p_x = x + width / 2;
        let p_z = z + self.rng.gen_range(1, width - 1);
        self.room.set_block(
            x + width / 2,
            height - 1,
            p_z - 1,
            Block::StoneLabel { text: label },
        );
        if self.rng.gen_range(0, 2) == 0 {
            let platform_length = if width <= 6 {
                1
            } else {
                self.rng.gen_range(1, 3)
            };
            self.room
                .build_pillar_head(p_x, height - platform_length, p_z, platform_length);
            if self.rng.gen_range(0, 2) == 0 {
                self.room.build_stair(
                    p_x - platform_length,
                    height - 1,
                    p_z,
                    platform_length,
                    false,
                );
            } else {
                self.room.build_stair(
                    p_x + platform_length,
                    height - 1,
                    p_z,
                    platform_length,
                    true,
                );
            }
        }

        self.sides.push(PlatformSide {
            x,
            height,
            z1: z,
            z2: z + depth,
            left: true,
            used: vec![false; depth],
        });
        self.sides.push(PlatformSide {
            x: x + width,
            height,
            z1: z,
            z2: z + depth,
            left: false,
            used: vec![false; depth],
        });
        self.edges.push(PlatformEdge {
            z,
            x1: x,
            x2: x + width,
            height,
            top: true,
        });
        self.edges.push(PlatformEdge {
            z: z + depth,
            x1: x,
            x2: x + width,
            height,
            top: false,
        });
        self.room.place_player(p_x, height - 5, p_z);
    }

    fn extend_platform(&mut self) {
        let edge_index = self.rng.gen_range(0, self.edges.len());
        let edge = self.edges.swap_remove(edge_index);
        let x_bridge = self.rng.gen_range(edge.x1, edge.x2);
        let extent_left = self.rng.gen_range(2, 5);
        let extent_right = self.rng.gen_range(2, 5);
        let x1 = x_bridge - extent_left;
        let width = extent_left + extent_right + 1;
        let x2 = x1 + width;

        let new_height = if !edge.top {
            if edge.height < 3 {
                edge.height
            } else {
                edge.height - self.rng.gen_range(0, 3)
            }
        } else {
            if edge.height > 9 {
                edge.height
            } else {
                edge.height + self.rng.gen_range(0, 3)
            }
        };

        let depth = self.rng.gen_range(2, 12);
        let (z1, z2) = if edge.top {
            self.room.build_pillar(x_bridge, edge.height, edge.z - 1);
            if edge.height != new_height {
                let stair_length = new_height - edge.height;
                self.room
                    .build_pillar(x_bridge - stair_length - 1, new_height, edge.z - 1);
                self.room
                    .build_pillar(x_bridge + stair_length + 1, new_height, edge.z - 1);
                self.room.build_stair(
                    x_bridge - stair_length,
                    new_height - 1,
                    edge.z - 1,
                    stair_length,
                    false,
                );
                self.room.build_stair(
                    x_bridge + stair_length,
                    new_height - 1,
                    edge.z - 1,
                    stair_length,
                    true,
                );
            }
            (edge.z - 1 - depth, edge.z - 1)
        } else {
            self.room.build_pillar(x_bridge, edge.height, edge.z);
            if edge.height != new_height {
                let stair_length = edge.height - new_height;
                self.room
                    .build_pillar(x_bridge - stair_length - 1, new_height, edge.z);
                self.room
                    .build_pillar(x_bridge + stair_length + 1, new_height, edge.z);
                self.room
                    .build_stair(x_bridge - 1, edge.height - 1, edge.z, stair_length, true);
                self.room
                    .build_stair(x_bridge + 1, edge.height - 1, edge.z, stair_length, false);
            }
            (edge.z + 1, edge.z + 1 + depth)
        };
        self.room.carve_platform(new_height, x1, z1, x2, z2);
        if edge.top {
            self.edges.push(PlatformEdge {
                z: z1,
                x1,
                x2,
                height: new_height,
                top: true,
            })
        } else {
            self.edges.push(PlatformEdge {
                z: z2,
                x1,
                x2,
                height: new_height,
                top: false,
            })
        }
        self.sides.push(PlatformSide {
            x: x1,
            height: new_height,
            z1,
            z2,
            left: true,
            used: vec![false; depth],
        });
        self.sides.push(PlatformSide {
            x: x2,
            height: new_height,
            z1,
            z2,
            left: false,
            used: vec![false; depth],
        })
    }

    pub fn build_choice<
        M: ReadStateSpace + ReadChoiceLabels<E = Option<String>> + ReadValuations,
    >(
        &mut self,
        mdp: M,
        choice: ChoiceIndex<M::Ix>,
    ) {
        let mut placements = Vec::new();
        for (index, side) in self.sides.iter().enumerate() {
            for i in 2..(side.z2 - side.z1) - 2 {
                let mut all_clear = true;
                for j in -2..=2 {
                    if side.used[(i as i32 + j) as usize] {
                        all_clear = false;
                    }
                }
                if all_clear {
                    let x = if side.left { side.x - 1 } else { side.x };
                    placements.push((index, x, side.height, side.z1 + i, side.left))
                }
            }
        }
        if placements.is_empty() {
            self.extend_platform();
            self.build_choice(mdp, choice);
        } else {
            let pl_idx = self.rng.gen_range(0, placements.len());
            let (index, mut x, mut y, z, left) = placements[pl_idx];

            let label = mdp.choice_label(choice);
            let text = if let Some(label) = label {
                label.clone()
            } else {
                "".to_string()
            };
            if left {
                self.room.build_sign(x + 1, y - 1, z + 1, true, text);
            } else {
                self.room.build_sign(x - 2, y - 1, z + 1, false, text);
            }

            let mut components = Vec::new();
            if self.rng.gen_range(0, 3) == 0 {
                components.push(ChoicePathComponent::Platform)
            }
            if self.rng.gen_range(0, 3) == 0 {
                components.push(ChoicePathComponent::StairsUp);
                components.push(ChoicePathComponent::Platform)
            }
            components.push(ChoicePathComponent::Slide);
            components.push(ChoicePathComponent::Platform);
            let mut remaining_probability = 1.0;
            for branch in mdp.branches_of_choice(choice) {
                let prob = mdp.branch_probability(branch) as f32;
                let collapsing = if remaining_probability > 0.0 {
                    self.transition_rng.gen_range(0.0, remaining_probability) <= prob
                } else {
                    false
                };
                remaining_probability -= prob;
                let dest = mdp.branch_destination(branch);
                let val = mdp.state_valuation(dest).to_string();
                components.push(ChoicePathComponent::Bridge(
                    prob,
                    collapsing,
                    val,
                    dest.raw().as_usize(),
                ));
                components.push(ChoicePathComponent::Platform);
                if self.rng.gen_range(0, 5) == 0 {
                    components.push(ChoicePathComponent::Platform);
                }
                if self.rng.gen_range(0, 3) == 0 {
                    components.push(ChoicePathComponent::StairsUp);
                    components.push(ChoicePathComponent::Platform);
                } else if self.rng.gen_range(0, 3) == 0 {
                    components.push(ChoicePathComponent::StairsDown);
                    components.push(ChoicePathComponent::Platform);
                }
            }

            let used_idx = self.sides[index].z1;
            self.sides[index].used[z - used_idx] = true;
            for component in components {
                match component {
                    ChoicePathComponent::Slide => {
                        let length = self.rng.gen_range(1, 4);
                        if left {
                            self.room
                                .build_slide(x - length + 1, y + length - 1, z, length, !left);
                            x -= length;
                            y += length;
                        } else {
                            self.room
                                .build_slide(x + length - 1, y + length - 1, z, length, !left);
                            x += length;
                            y += length;
                        }
                    }
                    ChoicePathComponent::StairsUp => {
                        let length = self.rng.gen_range(1, 4);
                        if left {
                            self.room.build_stair(x, y - 1, z, length, left);
                            x -= length;
                            y -= length;
                        } else {
                            self.room.build_stair(x, y - 1, z, length, left);
                            x += length;
                            y -= length;
                        }
                    }
                    ChoicePathComponent::StairsDown => {
                        let length = self.rng.gen_range(1, 4);
                        if left {
                            self.room
                                .build_stair(x - length + 1, y + length - 1, z, length, !left);
                            x -= length;
                            y += length;
                        } else {
                            self.room
                                .build_stair(x + length - 1, y + length - 1, z, length, !left);
                            x += length;
                            y += length;
                        }
                    }
                    ChoicePathComponent::Platform => {
                        self.room.build_pillar(x, y, z);
                        if left {
                            x -= 1;
                        } else {
                            x += 1;
                        }
                    }
                    ChoicePathComponent::Bridge(prob, collapsing, text, destination) => {
                        let half_length = self.rng.gen_range(0, 3);
                        let length = 3 + 2 * half_length;
                        let start_x = if left { x - length + 1 } else { x };
                        self.room
                            .build_bridge(start_x, y, z, length, prob, collapsing);
                        self.room
                            .build_tube(start_x + 1 + half_length, y + 1, z, text);
                        self.room.teleport_zones.push((
                            AABB::from_center_and_size(
                                Vec3::new(
                                    (start_x + half_length) as f32 + 1.5,
                                    y as f32 + 1.0,
                                    z as f32 + 0.5,
                                ),
                                Vec3::new(1.8, 1.0, 1.8),
                            ),
                            destination,
                        ));
                        if left {
                            x -= length;
                        } else {
                            x += length;
                        }
                    }
                }
            }
        }
    }
}

enum ChoicePathComponent {
    Slide,
    StairsUp,
    StairsDown,
    Platform,
    Bridge(f32, bool, String, usize), // Probability, collapsing, label, destination index of tube // Yes, I should just name these fields
}

pub struct Camera {
    pub scale: f32,
    pub offset: Vec2,
}

impl Camera {
    pub fn project_ints(&self, x: usize, y: isize, z: usize) -> Vec2 {
        self.project(Vec3::new(x as f32, y as f32, z as f32))
    }
    pub fn project(&self, position: Vec3) -> Vec2 {
        let x = position.x * self.scale - self.offset.x;
        let y = (position.y - position.z) * self.scale - self.offset.y;
        Vec2::new(x, y)
    }

    pub fn project_size(&self, size: Vec2) -> Vec2 {
        size * self.scale
    }

    pub fn update(&mut self, screen_size: (f32, f32), target: Vec3) {
        self.scale = screen_size.0.min(screen_size.1) / 12.0;
        let (tx, ty) = (target.x, target.y - target.z);
        let (tsx, tsy) = (tx * self.scale, ty * self.scale);
        self.offset = Vec2::new(tsx - screen_size.0 * 0.5, tsy - screen_size.1 * 0.5);
    }
}
