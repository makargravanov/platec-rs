// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geometry::{FloatVector, IntPoint, WorldDimension};
use super::mass::Mass;
use super::random::SimpleRandom;

const INITIAL_SPEED_X: f32 = 1.0;
const DEFORMATION_WEIGHT: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Movement {
    dimensions: WorldDimension,
    velocity: f32,
    rot_dir: f32,
    dx: f32,
    dy: f32,
    vx: f32,
    vy: f32,
}

impl Movement {
    pub fn new(mut random: SimpleRandom, dimensions: WorldDimension) -> Self {
        let mut stored_random = random;
        let rot_dir = if random.next_u32() % 2 != 0 {
            1.0
        } else {
            -1.0
        };
        let angle = 2.0 * core::f32::consts::PI * stored_random.next_f32();
        Self {
            dimensions,
            velocity: 1.0,
            rot_dir,
            dx: 0.0,
            dy: 0.0,
            vx: angle.cos() * INITIAL_SPEED_X,
            vy: angle.sin() * INITIAL_SPEED_X,
        }
    }

    pub fn apply_friction(&mut self, deformed_mass: f32, mass: f32) {
        if mass == 0.0 {
            self.velocity = 0.0;
            return;
        }
        let vel_dec = (DEFORMATION_WEIGHT * deformed_mass / mass).min(self.velocity);
        self.velocity -= vel_dec;
    }

    pub fn move_step(&mut self) {
        self.vx += self.dx;
        self.vy += self.dy;
        self.dx = 0.0;
        self.dy = 0.0;

        let len = (self.vx * self.vx + self.vy * self.vy).sqrt();
        assert!(len > 0.0);
        self.vx /= len;
        self.vy /= len;
        self.velocity += len - 1.0;
        if self.velocity <= 0.0 {
            self.velocity = 0.0;
        }

        let world_avg_side = (self.dimensions.width() + self.dimensions.height()) / 2;
        let alpha = self.rot_dir * self.velocity / (world_avg_side as f32 * 0.33);
        let alpha_vel = alpha * self.velocity;
        let cos = alpha_vel.cos();
        let sin = alpha_vel.sin();
        let vx = self.vx * cos - self.vy * sin;
        let vy = self.vy * cos + self.vx * sin;
        self.vx = vx;
        self.vy = vy;
    }

    pub fn velocity_on_x(&self) -> f32 {
        self.vx * self.velocity
    }
    pub fn velocity_on_y(&self) -> f32 {
        self.vy * self.velocity
    }
    pub fn velocity_on_x_len(&self, length: f32) -> f32 {
        assert!(length >= 0.0);
        self.vx * length
    }
    pub fn velocity_on_y_len(&self, length: f32) -> f32 {
        assert!(length >= 0.0);
        self.vy * length
    }
    pub fn dot(&self, dx: f32, dy: f32) -> f32 {
        self.vx * dx + self.vy * dy
    }
    pub fn momentum(&self, mass: Mass) -> f32 {
        mass.mass() * self.velocity
    }
    pub const fn velocity(&self) -> f32 {
        self.velocity
    }
    pub const fn unit_vector(&self) -> FloatVector {
        FloatVector::new(self.vx, self.vy)
    }
    pub fn add_impulse(&mut self, impulse: FloatVector) {
        self.dx += impulse.x;
        self.dy += impulse.y;
    }
    pub fn dec_impulse(&mut self, impulse: FloatVector) {
        self.dx -= impulse.x;
        self.dy -= impulse.y;
    }
}

pub fn collide_movements(
    this_mass: Mass,
    this_movement: &mut Movement,
    other_mass: Mass,
    other_movement: &mut Movement,
    coll_mass: f32,
) {
    let coeff_rest = 0.0;
    let this_center: IntPoint = this_mass.center().into();
    let other_center: IntPoint = other_mass.center().into();
    let mass_centers_distance = other_center - this_center;
    let distance = mass_centers_distance.length();
    if distance <= 0.0 {
        return;
    }

    let collision_direction = FloatVector::new(
        mass_centers_distance.x as f32 / distance,
        mass_centers_distance.y as f32 / distance,
    );
    let relative_velocity = this_movement.unit_vector() - other_movement.unit_vector();
    let rel_dot_n = collision_direction.dot(relative_velocity);
    if rel_dot_n <= 0.0 {
        return;
    }

    let col_len = collision_direction.length();
    let denom = col_len * col_len * (1.0 / other_mass.mass() + 1.0 / coll_mass);
    let impulse = -(1.0 + coeff_rest) * rel_dot_n / denom;
    this_movement.add_impulse(collision_direction * (impulse / this_mass.mass()));
    other_movement.dec_impulse(collision_direction * (impulse / (coll_mass + other_mass.mass())));
}
