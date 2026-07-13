// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::bounds::Bounds;
use super::geology::PlateVelocity;
use super::geometry::{FloatPoint, WorldDimension};
use super::mass::{Mass, MassBuilder};
use super::matrix::{AgeMap, HeightMap};
use super::movement::{Movement, collide_movements};
use super::plate_functions::calculate_crust;
use super::random::SimpleRandom;
use super::rectangle::BAD_INDEX;
use super::segments::{ContinentId, Segments};

#[derive(Debug, Clone, PartialEq)]
pub struct Plate {
    dimensions: WorldDimension,
    random: SimpleRandom,
    map: HeightMap,
    age_map: AgeMap,
    bounds: Bounds,
    mass: Mass,
    movement: Movement,
    segments: Segments,
}

impl Plate {
    pub fn new(
        seed: u32,
        map: Vec<f32>,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        plate_age: u32,
        dimensions: WorldDimension,
    ) -> Self {
        let mut ages = AgeMap::new(width, height, 0);
        for (index, value) in map.iter().copied().enumerate() {
            ages.as_mut_slice()[index] = if value > 0.0 { plate_age } else { 0 };
        }
        let random = SimpleRandom::new(seed);
        let mass = MassBuilder::from_map(&map, width, height).build();

        Self {
            dimensions,
            random,
            map: HeightMap::from_vec(width, height, map),
            age_map: ages,
            bounds: Bounds::new(
                dimensions,
                FloatPoint::new(x as f32, y as f32),
                width,
                height,
            ),
            mass,
            movement: Movement::new(random, dimensions),
            segments: Segments::new(width * height),
        }
    }

    pub fn add_collision(&mut self, wx: u32, wy: u32) -> u32 {
        let segment = self.continent_at(wx, wy);
        self.segments.data_mut(segment).inc_coll_count();
        self.segments.data(segment).area()
    }

    pub fn add_crust_by_collision(
        &mut self,
        x: u32,
        y: u32,
        z: f32,
        time: u32,
        active_continent: ContinentId,
    ) {
        let current = self.get_crust(x, y);
        self.set_crust(x, y, current + z, time);
        let mut lx = x;
        let mut ly = y;
        let index = self.bounds.get_valid_map_index(&mut lx, &mut ly) as usize;
        self.segments.set_id(index, active_continent);
        let data = self.segments.data_mut(active_continent);
        data.inc_area();
        data.enlarge_to_contain(lx, ly);
    }

    pub fn add_crust_by_subduction(
        &mut self,
        x: u32,
        y: u32,
        z: f32,
        mut time: u32,
        mut dx: f32,
        mut dy: f32,
    ) {
        let mut lx = x;
        let mut ly = y;
        self.bounds.get_valid_map_index(&mut lx, &mut ly);

        let dot = self.movement.dot(dx, dy);
        dx -= self.movement.velocity_on_x_len((dot > 0.0) as u32 as f32);
        dy -= self.movement.velocity_on_y_len((dot > 0.0) as u32 as f32);

        let mut offset = self.random.next_f32();
        let offset_sign = (2 * (self.random.next_u32() % 2) as i32 - 1) as f32;
        offset *= offset * offset * offset_sign;
        let mut offset2 = self.random.next_f32();
        let offset_sign2 = (2 * (self.random.next_u32() % 2) as i32 - 1) as f32;
        offset2 *= offset2 * offset2 * offset_sign2;
        dx = 10.0 * dx + 3.0 * offset;
        dy = 10.0 * dy + 3.0 * offset2;

        let fx = lx as f32 + dx;
        let fy = ly as f32 + dy;
        if self.bounds.is_in_limits(fx, fy) {
            let index = self.bounds.index(fx as u32, fy as u32) as usize;
            if self.map[index] > 0.0 {
                time = ((self.map[index] * self.age_map[index] as f32 + z * time as f32)
                    / (self.map[index] + z)) as u32;
                self.age_map.as_mut_slice()[index] = if z > 0.0 { time } else { 0 };
                self.map.as_mut_slice()[index] += z;
                self.mass.inc_mass(z);
            }
        }
    }

    pub fn aggregate_crust(&mut self, destination: &mut Plate, wx: u32, wy: u32) -> f32 {
        let mut lx = wx;
        let mut ly = wy;
        let index = self.bounds.get_valid_map_index(&mut lx, &mut ly) as usize;
        let segment_id = self.segments.id(index);
        if self.segments.data(segment_id).is_empty() {
            return 0.0;
        }

        let active_continent = destination.select_collision_segment(wx, wy);
        let shifted_wx = wx + self.dimensions.width();
        let shifted_wy = wy + self.dimensions.height();
        let old_mass = self.mass.mass();
        let top = self.segments.data(segment_id).top();
        let bottom = self.segments.data(segment_id).bottom();
        let left = self.segments.data(segment_id).left();
        let right = self.segments.data(segment_id).right();

        for y in top..=bottom {
            for x in left..=right {
                let i = (y * self.bounds.width() + x) as usize;
                if self.segments.id(i) == segment_id && self.map[i] > 0.0 {
                    destination.add_crust_by_collision(
                        shifted_wx + x - lx,
                        shifted_wy + y - ly,
                        self.map[i],
                        self.age_map[i],
                        active_continent,
                    );
                    self.mass.inc_mass(-self.map[i]);
                    self.map.as_mut_slice()[i] = 0.0;
                }
            }
        }

        self.segments.data_mut(segment_id).mark_non_existent();
        old_mass - self.mass.mass()
    }

    pub fn apply_friction(&mut self, deformed_mass: f32) {
        if !self.mass.is_null() {
            self.movement
                .apply_friction(deformed_mass, self.mass.mass());
        }
    }

    pub fn collide(&mut self, other: &mut Plate, coll_mass: f32) {
        if !self.mass.is_null() && coll_mass > 0.0 {
            collide_movements(
                self.mass,
                &mut self.movement,
                other.mass,
                &mut other.movement,
                coll_mass,
            );
        }
    }

    pub fn erode(&mut self, lower_bound: f32) {
        let mut sources = Vec::new();
        self.find_river_sources(lower_bound, &mut sources);
        let mut tmp = self.map.clone();
        self.flow_rivers(lower_bound, &mut sources, &mut tmp);

        for value in tmp.as_mut_slice() {
            let alpha = 0.2 * self.random.next_f32();
            *value += 0.1 * *value - alpha * *value;
            if *value < 0.0 {
                *value = 0.0;
            }
        }

        self.map = tmp.clone();
        tmp.as_mut_slice().fill(0.0);
        let mut mass_builder = MassBuilder::new();

        for y in 0..self.bounds.height() {
            for x in 0..self.bounds.width() {
                let index = (y * self.bounds.width() + x) as usize;
                mass_builder.add_point(x, y, self.map[index]);
                tmp.as_mut_slice()[index] += self.map[index];
                if self.map[index] < lower_bound {
                    continue;
                }
                let (crust, neighbor) = self.calculate_crust(x, y, index);
                if crust.iter().sum::<f32>() == 0.0 {
                    continue;
                }

                let diff = [
                    self.map[index] - crust[0],
                    self.map[index] - crust[1],
                    self.map[index] - crust[2],
                    self.map[index] - crust[3],
                ];
                let mut min_diff = diff[0].min(diff[1]).min(diff[2]).min(diff[3]);
                let diff_sum = (diff[0] - min_diff) * (crust[0] > 0.0) as u32 as f32
                    + (diff[1] - min_diff) * (crust[1] > 0.0) as u32 as f32
                    + (diff[2] - min_diff) * (crust[2] > 0.0) as u32 as f32
                    + (diff[3] - min_diff) * (crust[3] > 0.0) as u32 as f32;
                assert!(diff_sum >= 0.0);

                if diff_sum < min_diff {
                    for n in 0..4 {
                        tmp.as_mut_slice()[neighbor[n]] +=
                            (diff[n] - min_diff) * (crust[n] > 0.0) as u32 as f32;
                    }
                    tmp.as_mut_slice()[index] -= min_diff;
                    min_diff -= diff_sum;
                    let count = 1
                        + (crust[0] > 0.0) as u32
                        + (crust[1] > 0.0) as u32
                        + (crust[2] > 0.0) as u32
                        + (crust[3] > 0.0) as u32;
                    min_diff /= count as f32;
                    for n in 0..4 {
                        tmp.as_mut_slice()[neighbor[n]] +=
                            min_diff * (crust[n] > 0.0) as u32 as f32;
                    }
                    tmp.as_mut_slice()[index] += min_diff;
                } else {
                    let unit = min_diff / diff_sum;
                    tmp.as_mut_slice()[index] -= min_diff;
                    for n in 0..4 {
                        tmp.as_mut_slice()[neighbor[n]] +=
                            unit * (diff[n] - min_diff) * (crust[n] > 0.0) as u32 as f32;
                    }
                }
            }
        }

        for value in tmp.as_mut_slice() {
            if *value < 0.0 {
                *value = 0.0;
            }
        }
        self.map = tmp;
        self.mass = mass_builder.build();
    }

    pub fn get_collision_info(&mut self, wx: u32, wy: u32) -> (u32, f32) {
        let segment = self.continent_at(wx, wy);
        let data = self.segments.data(segment);
        (
            data.coll_count(),
            data.coll_count() as f32 / (1 + data.area()) as f32,
        )
    }

    pub fn get_crust(&self, mut x: u32, mut y: u32) -> f32 {
        let index = self.bounds.get_map_index(&mut x, &mut y);
        if index != BAD_INDEX {
            self.map[index as usize]
        } else {
            0.0
        }
    }

    pub fn get_crust_timestamp(&self, mut x: u32, mut y: u32) -> u32 {
        let index = self.bounds.get_map_index(&mut x, &mut y);
        if index != BAD_INDEX {
            self.age_map[index as usize]
        } else {
            0
        }
    }

    pub fn move_step(&mut self) {
        self.movement.move_step();
        self.bounds
            .shift(self.movement.velocity_on_x(), self.movement.velocity_on_y());
    }

    pub fn reset_segments(&mut self) {
        assert_eq!(self.bounds.area(), self.segments.area());
        self.segments.reset();
    }

    pub fn set_crust(&mut self, mut x: u32, mut y: u32, mut z: f32, mut time: u32) {
        if z < 0.0 {
            z = 0.0;
        }

        let mut lx = x;
        let mut ly = y;
        let mut index = self.bounds.get_map_index(&mut lx, &mut ly);
        if index == BAD_INDEX {
            assert!(z > 0.0);
            let ilft = self.bounds.left();
            let itop = self.bounds.top();
            let irgt = self.bounds.right_non_inclusive();
            let ibtm = self.bounds.bottom_non_inclusive();
            self.dimensions.normalize(&mut x, &mut y);

            let left_dist = ilft.wrapping_sub(x);
            let right_dist = (self.dimensions.width() & mask(x < ilft))
                .wrapping_add(x)
                .wrapping_sub(irgt);
            let top_dist = itop.wrapping_sub(y);
            let bottom_dist = (self.dimensions.height() & mask(y < itop))
                .wrapping_add(y)
                .wrapping_sub(ibtm);

            let mut d_left = left_dist
                & mask(left_dist < right_dist)
                & mask(left_dist < self.dimensions.width());
            let mut d_right = right_dist
                & mask(right_dist <= left_dist)
                & mask(right_dist < self.dimensions.width());
            let mut d_top =
                top_dist & mask(top_dist < bottom_dist) & mask(top_dist < self.dimensions.height());
            let mut d_bottom = bottom_dist
                & mask(bottom_dist <= top_dist)
                & mask(bottom_dist < self.dimensions.height());

            d_left = (((d_left > 0) as u32) + (d_left >> 3)) << 3;
            d_right = (((d_right > 0) as u32) + (d_right >> 3)) << 3;
            d_top = (((d_top > 0) as u32) + (d_top >> 3)) << 3;
            d_bottom = (((d_bottom > 0) as u32) + (d_bottom >> 3)) << 3;

            if self.bounds.width() + d_left + d_right > self.dimensions.width() {
                d_left = 0;
                d_right = self.dimensions.width() - self.bounds.width();
            }
            if self.bounds.height() + d_top + d_bottom > self.dimensions.height() {
                d_top = 0;
                d_bottom = self.dimensions.height() - self.bounds.height();
            }
            assert!(d_left + d_right + d_top + d_bottom != 0);

            let old_width = self.bounds.width();
            let old_height = self.bounds.height();
            self.bounds.shift(-(d_left as f32), -(d_top as f32));
            self.bounds.grow(d_left + d_right, d_top + d_bottom);

            let mut new_map = HeightMap::new(self.bounds.width(), self.bounds.height(), 0.0);
            let mut new_age = AgeMap::new(self.bounds.width(), self.bounds.height(), 0);
            let mut new_segments = vec![u32::MAX; self.bounds.area() as usize];
            for row in 0..old_height {
                let dest = ((d_top + row) * self.bounds.width() + d_left) as usize;
                let src = (row * old_width) as usize;
                let len = old_width as usize;
                new_map.as_mut_slice()[dest..dest + len]
                    .copy_from_slice(&self.map.as_slice()[src..src + len]);
                new_age.as_mut_slice()[dest..dest + len]
                    .copy_from_slice(&self.age_map.as_slice()[src..src + len]);
                new_segments[dest..dest + len]
                    .copy_from_slice(&self.segments.ids()[src..src + len]);
            }
            self.map = new_map;
            self.age_map = new_age;
            self.segments.reassign(new_segments);
            self.segments.shift(d_left, d_top);

            lx = x;
            ly = y;
            index = self.bounds.get_valid_map_index(&mut lx, &mut ly);
        }

        let index = index as usize;
        if self.map[index] > 0.0 {
            time = ((self.map[index] * self.age_map[index] as f32 + z * time as f32)
                / (self.map[index] + z)) as u32;
        }
        if z > 0.0 {
            self.age_map.as_mut_slice()[index] = time;
        }
        self.mass.inc_mass(-self.map[index]);
        self.mass.inc_mass(z);
        self.map.as_mut_slice()[index] = z;
    }

    pub fn select_collision_segment(&mut self, x: u32, y: u32) -> ContinentId {
        let mut lx = x;
        let mut ly = y;
        let index = self.bounds.get_valid_map_index(&mut lx, &mut ly);
        self.segments.id(index as usize)
    }

    pub fn left(&self) -> u32 {
        self.bounds.left()
    }
    pub fn top(&self) -> u32 {
        self.bounds.top()
    }
    pub fn width(&self) -> u32 {
        self.bounds.width()
    }
    pub fn height(&self) -> u32 {
        self.bounds.height()
    }
    pub fn map(&self) -> &[f32] {
        self.map.as_slice()
    }
    pub fn age_map(&self) -> &[u32] {
        self.age_map.as_slice()
    }
    pub fn age_map_mut(&mut self) -> &mut [u32] {
        self.age_map.as_mut_slice()
    }
    pub fn velocity(&self) -> f32 {
        self.movement.velocity()
    }
    pub fn momentum(&self) -> f32 {
        self.movement.momentum(self.mass)
    }
    pub fn velocity_vector(&self) -> PlateVelocity {
        PlateVelocity::new(self.movement.velocity_on_x(), self.movement.velocity_on_y())
    }
    pub fn vel_x(&self) -> f32 {
        self.movement.unit_vector().x
    }
    pub fn vel_y(&self) -> f32 {
        self.movement.unit_vector().y
    }

    fn continent_at(&mut self, x: u32, y: u32) -> ContinentId {
        self.segments
            .get_continent_at(x, y, &self.bounds, self.map.as_slice(), self.dimensions)
    }

    fn calculate_crust(&self, x: u32, y: u32, index: usize) -> ([f32; 4], [usize; 4]) {
        calculate_crust(
            x,
            y,
            index,
            self.dimensions,
            self.map.as_slice(),
            self.bounds.width(),
            self.bounds.height(),
        )
    }

    fn find_river_sources(&self, lower_bound: f32, sources: &mut Vec<u32>) {
        for y in 0..self.bounds.height() {
            for x in 0..self.bounds.width() {
                let index = y * self.bounds.width() + x;
                if self.map[index as usize] < lower_bound {
                    continue;
                }
                let (crust, _) = self.calculate_crust(x, y, index as usize);
                if crust[0] * crust[1] * crust[2] * crust[3] == 0.0 {
                    continue;
                }
                sources.push(index);
            }
        }
    }

    fn flow_rivers(&self, lower_bound: f32, sources: &mut Vec<u32>, tmp: &mut HeightMap) {
        let mut sinks = Vec::new();
        let mut flow_done = vec![false; self.bounds.area() as usize];

        while !sources.is_empty() {
            while let Some(index) = sources.pop() {
                let y = index / self.bounds.width();
                let x = index - y * self.bounds.width();
                if self.map[index as usize] < lower_bound {
                    continue;
                }
                let (mut crust, neighbor) = self.calculate_crust(x, y, index as usize);
                if crust.iter().sum::<f32>() == 0.0 {
                    continue;
                }
                for item in &mut crust {
                    if *item == 0.0 {
                        *item = self.map[index as usize];
                    }
                }
                let mut lowest = crust[0];
                let mut dest = index.wrapping_sub(1);
                if crust[1] < lowest {
                    lowest = crust[1];
                    dest = index + 1;
                }
                if crust[2] < lowest {
                    lowest = crust[2];
                    dest = index.wrapping_sub(self.bounds.width());
                }
                if crust[3] < lowest {
                    dest = index + self.bounds.width();
                }

                if (dest as usize) < flow_done.len() && !flow_done[dest as usize] {
                    sinks.push(dest);
                    flow_done[dest as usize] = true;
                }
                tmp.as_mut_slice()[index as usize] -=
                    (tmp.as_slice()[index as usize] - lower_bound) * 0.2;

                let _ = neighbor;
            }
            core::mem::swap(sources, &mut sinks);
            sinks.clear();
        }
    }
}

fn mask(value: bool) -> u32 {
    if value { u32::MAX } else { 0 }
}
