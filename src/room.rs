use macroquad::color::{BLACK, Color};
use macroquad::input::{KeyCode, is_key_down};
use macroquad::math::Rect;
use macroquad::prelude::{DrawTextureParams, Texture2D, Vec2, Vec3, draw_texture_ex};
use macroquad::time::get_frame_time;

#[derive(Clone)]
pub struct Entity {
    position: Vec3,
    size: Vec2,
    color: Color,
}

impl Entity {
    pub fn draw(&self, camera: &Camera, texture: &Texture2D) {
        let position = camera.project(self.position);
        let size = camera.project_size(self.size);
        let (tx, ty) = (0, 0);
        let rect = Rect::new(16.0 * (tx as f32), 16.0 * (ty as f32), 16.0, 16.0);
        draw_texture_ex(
            texture,
            position.x - size.x * 0.5,
            position.y - size.y * 0.5,
            self.color,
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

pub struct Player {
    position: Vec3,
    velocity: Vec3,
    width: f32,
    height: f32,
}

pub enum Block {
    Air,
    SolidAir,
    Floor,
    Pillar { segment: u8 },
    Bridge { quality: u8, segment: u8 },
    StairLeft,
    StairRight,
    SlideLeft,
    SlideRight,
    Sign { left: bool, segment: u8 },
    Tube { segment: u8 },
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
                panic!("Cannot render slides yet");
            }
            Block::SlideRight => {
                panic!("Cannot render slides yet");
            }
            Block::Sign { left, segment } => {
                let start = if *left { 5 } else { 3 };
                Self::draw_internal(x, y, z, start + *segment as usize, 2, camera, texture);
            }
            Block::Tube { segment } => {
                Self::draw_internal(x, y - 1, z, 0, 2 + *segment as usize, camera, texture);
            }
        }
    }

    pub fn intersects(&self, x: usize, y: usize, z: usize, other: &AABB) -> bool {
        match self {
            Block::Air | Block::SolidAir => false,
            Block::Floor | Block::Pillar { .. } => AABB::int_cube(x, y, z).intersects(other),
            Block::Bridge { segment, .. } => {
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
                            Vec3::new(x as f32 + i as f32 * seg_size, y as f32 + height, z as f32),
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
            Block::StairLeft | Block::StairRight => {
                let segs = 32;
                let seg_size = 1.0 / segs as f32;
                for i in 0..32 {
                    let height = if matches!(self, Block::StairLeft) {
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
            Block::SlideLeft => false,
            Block::SlideRight => false,
            Block::Sign { .. } => AABB::from_pos_and_size(
                Vec3::new(x as f32, y as f32, z as f32),
                Vec3::new(1.0, 1.0, 0.2),
            )
            .intersects(other),
            Block::Tube { .. } => false,
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

    pub fn intersects(&self, other: &AABB) -> bool {
        ((self.min.x >= other.min.x && self.min.x <= other.max.x)
            || (other.min.x >= self.min.x && other.min.x <= self.max.x))
            && ((self.min.y >= other.min.y && self.min.y <= other.max.y)
                || (other.min.y >= self.min.y && other.min.y <= self.max.y))
            && ((self.min.z >= other.min.z && self.min.z <= other.max.z)
                || (other.min.z >= self.min.z && other.min.z <= self.max.z))
    }
}

pub struct Room {
    player: Player,
    blocks: Vec<Vec<Vec<Block>>>,
    entities_per_layer: Vec<Vec<Entity>>,
    width: usize,
    height: usize,
    depth: usize,
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
                width: 0.6,
                height: 0.6 + 1.0,
            },
            blocks,
            entities_per_layer: vec![Vec::new(); depth],
            width,
            depth,
            height,
        }
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[z][y][x]
    }
    fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[z][y][x] = block;
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

    fn insert_entity(&mut self, position: Vec3, size: Vec2, color: Color) {
        let layer = (position.z).clamp(0.0, self.entities_per_layer.len() as f32 - 0.05) as usize;
        self.entities_per_layer[layer].push(Entity {
            position,
            size,
            color,
        })
    }

    pub fn draw(&mut self, texture: &Texture2D, camera: &Camera) {
        for layer in &mut self.entities_per_layer {
            layer.clear();
        }
        self.insert_entity(
            self.player.position + Vec3::new(0.0, self.player.height * -0.5, 0.0),
            Vec2::new(self.player.width, self.player.height),
            BLACK,
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
        let elapsed = get_frame_time();
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
        self.player.velocity.y += 50.0 * elapsed;

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

    pub fn build_bridge(
        &mut self,
        x: usize,
        height: usize,
        z: usize,
        length: usize,
        probability: f32,
    ) {
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
    pub fn build_tube(&mut self, x: usize, height: usize, z: usize) {
        for i in 0..5 {
            self.set_block(x, height + i, z, Block::Tube { segment: i as u8 });
        }
    }
    pub fn build_sign(&mut self, x: usize, height: usize, z: usize, left: bool) {
        for i in 0..2 {
            self.set_block(
                x + i,
                height,
                z,
                Block::Sign {
                    left,
                    segment: i as u8,
                },
            );
        }
    }
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
        let x = (position.x * self.scale - self.offset.x);
        let y = (position.y - position.z) * self.scale - self.offset.y;
        Vec2::new(x, y)
    }

    pub fn project_size(&self, size: Vec2) -> Vec2 {
        size * self.scale
    }

    pub fn update(&mut self, screen_size: (f32, f32), target: Vec3) {
        self.scale = screen_size.0.min(screen_size.1) / 8.0; //16.0;
        let (tx, ty) = (target.x, target.y - target.z);
        let (tsx, tsy) = (tx * self.scale, ty * self.scale);
        self.offset = Vec2::new(tsx - screen_size.0 * 0.5, tsy - screen_size.1 * 0.5);
    }
}
