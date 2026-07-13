// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geology::PlateVelocity;
use super::geometry::WorldDimension;
use super::matrix::{AgeMap, HeightMap, IndexMap};
use super::plate::Plate;
use super::random::SimpleRandom;
use super::simplexnoise::scaled_octave_noise_4d;

const CONTINENTAL_BASE: f32 = 1.0;
const OCEANIC_BASE: f32 = 0.1;
const SUBDUCT_RATIO: f32 = 0.5;
const BUOYANCY_BONUS_X: f32 = 3.0;
const MAX_BUOYANCY_AGE: u32 = 20;
const MULINV_MAX_BUOYANCY_AGE: f32 = 1.0 / MAX_BUOYANCY_AGE as f32;
const RESTART_ENERGY_RATIO: f32 = 0.15;
const RESTART_SPEED_LIMIT: f32 = 2.0;
const RESTART_ITERATIONS: u32 = 600;
const NO_COLLISION_TIME_LIMIT: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatecConfig {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub sea_level: f32,
    pub erosion_period: u32,
    pub folding_ratio: f32,
    pub aggregation_overlap_abs: u32,
    pub aggregation_overlap_rel: f32,
    pub cycle_count: u32,
    pub plate_count: u32,
}

impl Default for PlatecConfig {
    fn default() -> Self {
        Self {
            seed: 3,
            width: 600,
            height: 400,
            sea_level: 0.65,
            erosion_period: 60,
            folding_ratio: 0.02,
            aggregation_overlap_abs: 1_000_000,
            aggregation_overlap_rel: 0.33,
            cycle_count: 2,
            plate_count: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatecError {
    DimensionsTooSmall,
    ZeroPlates,
}

#[derive(Debug, Clone)]
pub struct Lithosphere {
    dimensions: WorldDimension,
    hmap: HeightMap,
    imap: IndexMap,
    prev_imap: IndexMap,
    amap: AgeMap,
    plates: Vec<Plate>,
    plate_areas: Vec<PlateArea>,
    plate_indices_found: Vec<u32>,
    collisions: Vec<Vec<PlateCollision>>,
    subductions: Vec<Vec<PlateCollision>>,
    random: SimpleRandom,
    config: PlatecConfig,
    plate_velocities: Vec<PlateVelocity>,
    iter_count: u32,
    peak_energy: f32,
    last_collision_count: u32,
    cycle_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlateCollision {
    index: u32,
    x: u32,
    y: u32,
    crust: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PlateArea {
    border: Vec<u32>,
    bottom: u32,
    left: u32,
    right: u32,
    top: u32,
    width: u32,
    height: u32,
}

impl Lithosphere {
    pub fn new(config: PlatecConfig) -> Result<Self, PlatecError> {
        if config.width < 5 || config.height < 5 {
            return Err(PlatecError::DimensionsTooSmall);
        }
        if config.plate_count == 0 {
            return Err(PlatecError::ZeroPlates);
        }

        let dimensions = WorldDimension::new(config.width, config.height);
        let mut random = SimpleRandom::new(config.seed);
        let hmap = create_initial_heightmap(dimensions, config.sea_level, &mut random);
        let mut lithosphere = Self {
            dimensions,
            hmap,
            imap: IndexMap::new(config.width, config.height, config.plate_count),
            prev_imap: IndexMap::new(config.width, config.height, config.plate_count),
            amap: AgeMap::new(config.width, config.height, 0),
            plates: Vec::new(),
            plate_areas: vec![PlateArea::default(); config.plate_count as usize],
            plate_indices_found: vec![0; config.plate_count as usize],
            collisions: vec![Vec::new(); config.plate_count as usize],
            subductions: vec![Vec::new(); config.plate_count as usize],
            random,
            config,
            plate_velocities: Vec::new(),
            iter_count: 0,
            peak_energy: 0.0,
            last_collision_count: 0,
            cycle_count: 0,
        };
        lithosphere.create_plates();
        Ok(lithosphere)
    }

    #[inline]
    pub const fn width(&self) -> u32 {
        self.dimensions.width()
    }

    #[inline]
    pub const fn height(&self) -> u32 {
        self.dimensions.height()
    }

    #[inline]
    pub fn heightmap(&self) -> &[f32] {
        self.hmap.as_slice()
    }

    #[inline]
    pub fn plates_map(&self) -> &[u32] {
        self.imap.as_slice()
    }

    #[inline]
    pub fn age_map(&self) -> &[u32] {
        self.amap.as_slice()
    }

    #[inline]
    pub fn plate_velocities(&self) -> &[PlateVelocity] {
        &self.plate_velocities
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.plates.is_empty()
    }

    pub fn step(&mut self) {
        if self.plates.is_empty() {
            return;
        }

        let mut total_velocity = 0.0;
        let mut system_kinetic_energy = 0.0;
        for plate in &self.plates {
            total_velocity += plate.velocity();
            system_kinetic_energy += plate.momentum();
        }
        if system_kinetic_energy > self.peak_energy {
            self.peak_energy = system_kinetic_energy;
        }
        if total_velocity < RESTART_SPEED_LIMIT
            || system_kinetic_energy / self.peak_energy < RESTART_ENERGY_RATIO
            || self.last_collision_count > NO_COLLISION_TIME_LIMIT
            || self.iter_count > RESTART_ITERATIONS
        {
            self.restart();
            return;
        }

        self.prev_imap = self.imap.clone();
        for plate in &mut self.plates {
            plate.reset_segments();
            if self.config.erosion_period > 0 && self.iter_count % self.config.erosion_period == 0 {
                plate.erode(CONTINENTAL_BASE);
            }
            plate.move_step();
        }

        let mut oceanic_collisions = 0;
        let mut continental_collisions = 0;
        self.update_height_and_plate_maps(&mut oceanic_collisions, &mut continental_collisions);
        self.last_collision_count =
            (self.last_collision_count + 1) & mask(continental_collisions == 0);

        for i in 0..self.plates.len() {
            let subductions = core::mem::take(&mut self.subductions[i]);
            for coll in subductions {
                assert_ne!(i as u32, coll.index);
                let dx = self.plates[coll.index as usize].vel_x();
                let dy = self.plates[coll.index as usize].vel_y();
                self.plates[i].add_crust_by_subduction(
                    coll.x,
                    coll.y,
                    coll.crust,
                    self.iter_count,
                    dx,
                    dy,
                );
            }
        }

        self.update_collisions();
        self.plate_indices_found.fill(0);

        for y in 0..self.dimensions.height() {
            for x in 0..self.dimensions.width() {
                let i = self.dimensions.index_of(x, y);
                if self.imap[i] >= self.plates.len() as u32 {
                    self.imap.as_mut_slice()[i] = self.prev_imap[i];
                    self.amap.as_mut_slice()[i] = self.iter_count;
                    self.hmap.as_mut_slice()[i] = OCEANIC_BASE * BUOYANCY_BONUS_X;
                    if self.imap[i] < self.plates.len() as u32 {
                        self.plates[self.imap[i] as usize].set_crust(
                            x,
                            y,
                            OCEANIC_BASE,
                            self.iter_count,
                        );
                    }
                } else {
                    let owner = self.imap[i] as usize;
                    self.plate_indices_found[owner] += 1;
                    assert!(self.hmap[i] > 0.0);
                }
            }
        }

        self.remove_empty_plates();

        for i in 0..self.dimensions.area() as usize {
            let mut crust_age = self.iter_count.wrapping_sub(self.amap[i]);
            crust_age = MAX_BUOYANCY_AGE.wrapping_sub(crust_age);
            crust_age &= mask(crust_age <= MAX_BUOYANCY_AGE);
            self.hmap.as_mut_slice()[i] += (self.hmap[i] < CONTINENTAL_BASE) as u32 as f32
                * BUOYANCY_BONUS_X
                * OCEANIC_BASE
                * crust_age as f32
                * MULINV_MAX_BUOYANCY_AGE;
        }

        self.refresh_plate_velocities();
        self.iter_count += 1;
    }

    pub fn plate_count(&self) -> u32 {
        self.plates.len() as u32
    }

    fn create_plates(&mut self) {
        let map_area = self.dimensions.area();
        let plate_count = self.config.plate_count;
        self.plates.clear();
        self.plate_areas
            .resize(plate_count as usize, PlateArea::default());

        for i in 0..map_area {
            self.imap.as_mut_slice()[i as usize] = i;
        }

        for i in 0..plate_count {
            let p = self.imap.as_slice()[(self.random.next_u32() % (map_area - i)) as usize];
            let y = self.dimensions.y_from_index(p);
            let x = self.dimensions.x_from_index(p);
            let area = &mut self.plate_areas[i as usize];
            area.left = x;
            area.right = x;
            area.top = y;
            area.bottom = y;
            area.width = 1;
            area.height = 1;
            area.border.clear();
            area.border.push(p);
            self.imap.as_mut_slice()[p as usize] =
                self.imap.as_slice()[(map_area - i - 1) as usize];
        }

        self.imap.as_mut_slice().fill(u32::MAX);
        self.grow_plates();

        for owner in self.imap.as_slice() {
            assert!(*owner < plate_count);
        }

        for i in 0..plate_count {
            let area = &mut self.plate_areas[i as usize];
            area.width = self.dimensions.x_cap(area.width);
            area.height = self.dimensions.y_cap(area.height);
            let x0 = area.left;
            let x1 = 1 + x0 + area.width;
            let y0 = area.top;
            let y1 = 1 + y0 + area.height;
            let width = x1 - x0;
            let height = y1 - y0;
            let mut plate_map = Vec::with_capacity((width * height) as usize);

            for y in y0..y1 {
                for x in x0..x1 {
                    let k = self.dimensions.normalized_index_of(x, y);
                    let value = self.hmap[k] * (self.imap[k] == i) as u32 as f32;
                    plate_map.push(value);
                }
            }

            self.plates.push(Plate::new(
                self.random.next_u32(),
                plate_map,
                width,
                height,
                x0,
                y0,
                i,
                self.dimensions,
            ));
        }

        self.refresh_plate_velocities();
        self.iter_count = plate_count + 20;
        self.peak_energy = 0.0;
        self.last_collision_count = 0;
        self.collisions.resize(plate_count as usize, Vec::new());
        self.subductions.resize(plate_count as usize, Vec::new());
        self.plate_indices_found.resize(plate_count as usize, 0);
    }

    fn grow_plates(&mut self) {
        let mut max_border = 1_u32;
        let plate_count = self.config.plate_count;
        while max_border != 0 {
            max_border = 0;
            for i in 0..plate_count {
                let area_index = i as usize;
                let border_len = self.plate_areas[area_index].border.len() as u32;
                max_border = max_border.max(border_len);
                if border_len == 0 {
                    continue;
                }

                let j = (self.random.next_u32() % border_len) as usize;
                let p = self.plate_areas[area_index].border[j];
                let cy = self.dimensions.y_from_index(p);
                let cx = self.dimensions.x_from_index(p);

                let left = if cx > 0 {
                    cx - 1
                } else {
                    self.dimensions.width() - 1
                };
                let right = if cx < self.dimensions.width() - 1 {
                    cx + 1
                } else {
                    0
                };
                let top = if cy > 0 {
                    cy - 1
                } else {
                    self.dimensions.height() - 1
                };
                let bottom = if cy < self.dimensions.height() - 1 {
                    cy + 1
                } else {
                    0
                };

                let north = top * self.dimensions.width() + cx;
                let south = bottom * self.dimensions.width() + cx;
                let west = cy * self.dimensions.width() + left;
                let east = cy * self.dimensions.width() + right;

                self.claim_plate_neighbor(i, north, |area, dimensions| {
                    if area.top == dimensions.y_mod(top + 1) {
                        area.top = top;
                        area.height += 1;
                    }
                });
                self.claim_plate_neighbor(i, south, |area, dimensions| {
                    if bottom == dimensions.y_mod(area.bottom + 1) {
                        area.bottom = bottom;
                        area.height += 1;
                    }
                });
                self.claim_plate_neighbor(i, west, |area, dimensions| {
                    if area.left == dimensions.x_mod(left + 1) {
                        area.left = left;
                        area.width += 1;
                    }
                });
                self.claim_plate_neighbor(i, east, |area, dimensions| {
                    if right == dimensions.x_mod(area.right + 1) {
                        area.right = right;
                        area.width += 1;
                    }
                });

                let area = &mut self.plate_areas[area_index];
                area.border[j] = *area.border.last().unwrap();
                area.border.pop();
            }
        }
    }

    fn claim_plate_neighbor(
        &mut self,
        plate_id: u32,
        index: u32,
        update_area: impl FnOnce(&mut PlateArea, WorldDimension),
    ) {
        if self.imap.as_slice()[index as usize] >= self.config.plate_count {
            self.imap.as_mut_slice()[index as usize] = plate_id;
            let area = &mut self.plate_areas[plate_id as usize];
            area.border.push(index);
            update_area(area, self.dimensions);
        }
    }

    fn update_height_and_plate_maps(
        &mut self,
        oceanic_collisions: &mut u32,
        continental_collisions: &mut u32,
    ) {
        let world_width = self.dimensions.width();
        let world_height = self.dimensions.height();
        self.hmap.as_mut_slice().fill(0.0);
        self.imap.as_mut_slice().fill(u32::MAX);

        for i in 0..self.plates.len() {
            let x0 = self.plates[i].left();
            let y0 = self.plates[i].top();
            let x1 = x0 + self.plates[i].width();
            let y1 = y0 + self.plates[i].height();
            let x_mod_start = (x0 + world_width) % world_width;
            let mut y_mod = (y0 + world_height) % world_height;
            let mut j = 0_usize;

            for _y in y0..y1 {
                let y_width = y_mod * world_width;
                let mut x_mod = x_mod_start;

                for _x in x0..x1 {
                    let k = (x_mod + y_width) as usize;
                    let mut this_height = self.plates[i].map()[j];
                    let mut this_age = self.plates[i].age_map()[j];

                    if this_height < 2.0 * f32::EPSILON {
                        j += 1;
                        x_mod += 1;
                        if x_mod >= world_width {
                            x_mod -= world_width;
                        }
                        continue;
                    }

                    if self.imap[k] >= self.plates.len() as u32 {
                        self.hmap.as_mut_slice()[k] = this_height;
                        self.imap.as_mut_slice()[k] = i as u32;
                        self.amap.as_mut_slice()[k] = this_age;
                        j += 1;
                        x_mod += 1;
                        if x_mod >= world_width {
                            x_mod -= world_width;
                        }
                        continue;
                    }

                    let prev_owner = self.imap[k] as usize;
                    let prev_is_oceanic = self.hmap[k] < CONTINENTAL_BASE;
                    let this_is_oceanic = this_height < CONTINENTAL_BASE;
                    let prev_timestamp = self.plates[prev_owner].get_crust_timestamp(x_mod, y_mod);
                    let prev_is_buoyant = self.hmap[k] > this_height
                        || (self.hmap[k] + 2.0 * f32::EPSILON > this_height
                            && self.hmap[k] < 2.0 * f32::EPSILON + this_height
                            && prev_timestamp >= this_age);

                    if this_is_oceanic && prev_is_buoyant {
                        let sediment =
                            SUBDUCT_RATIO * OCEANIC_BASE * (CONTINENTAL_BASE - this_height)
                                / CONTINENTAL_BASE;
                        self.subductions[prev_owner].push(PlateCollision {
                            index: i as u32,
                            x: x_mod,
                            y: y_mod,
                            crust: sediment,
                        });
                        *oceanic_collisions += 1;
                        self.plates[i].set_crust(
                            x_mod,
                            y_mod,
                            this_height - OCEANIC_BASE,
                            this_age,
                        );
                        this_height = self.plates[i].get_crust(x_mod, y_mod);
                        this_age = self.plates[i].get_crust_timestamp(x_mod, y_mod);
                        if this_height <= 0.0 {
                            j += 1;
                            x_mod += 1;
                            if x_mod >= world_width {
                                x_mod -= world_width;
                            }
                            continue;
                        }
                    } else if prev_is_oceanic {
                        let sediment =
                            SUBDUCT_RATIO * OCEANIC_BASE * (CONTINENTAL_BASE - self.hmap[k])
                                / CONTINENTAL_BASE;
                        self.subductions[i].push(PlateCollision {
                            index: prev_owner as u32,
                            x: x_mod,
                            y: y_mod,
                            crust: sediment,
                        });
                        *oceanic_collisions += 1;
                        self.plates[prev_owner].set_crust(
                            x_mod,
                            y_mod,
                            self.hmap[k] - OCEANIC_BASE,
                            prev_timestamp,
                        );
                        self.hmap.as_mut_slice()[k] -= OCEANIC_BASE;
                        if self.hmap[k] <= 0.0 {
                            self.imap.as_mut_slice()[k] = i as u32;
                            self.hmap.as_mut_slice()[k] = this_height;
                            self.amap.as_mut_slice()[k] = this_age;
                            j += 1;
                            x_mod += 1;
                            if x_mod >= world_width {
                                x_mod -= world_width;
                            }
                            continue;
                        }
                    }

                    self.resolve_juxtapositions(
                        i,
                        j,
                        k,
                        x_mod,
                        y_mod,
                        this_height,
                        this_age,
                        continental_collisions,
                    );

                    j += 1;
                    x_mod += 1;
                    if x_mod >= world_width {
                        x_mod -= world_width;
                    }
                }

                y_mod += 1;
                if y_mod >= world_height {
                    y_mod -= world_height;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_juxtapositions(
        &mut self,
        current: usize,
        local_index: usize,
        world_index: usize,
        x_mod: u32,
        y_mod: u32,
        this_height: f32,
        this_age: u32,
        continental_collisions: &mut u32,
    ) {
        let previous = self.imap[world_index] as usize;
        let (this_area, prev_area) = {
            let (current_plate, previous_plate) = two_mut(&mut self.plates, current, previous);
            (
                current_plate.add_collision(x_mod, y_mod),
                previous_plate.add_collision(x_mod, y_mod),
            )
        };

        if this_area < prev_area {
            let crust = this_height * self.config.folding_ratio;
            self.hmap.as_mut_slice()[world_index] += crust;
            self.plates[previous].set_crust(x_mod, y_mod, self.hmap[world_index], this_age);
            self.plates[current].set_crust(
                x_mod,
                y_mod,
                this_height * (1.0 - self.config.folding_ratio),
                this_age,
            );
            self.collisions[current].push(PlateCollision {
                index: previous as u32,
                x: x_mod,
                y: y_mod,
                crust,
            });
            *continental_collisions += 1;
        } else {
            let crust = self.hmap[world_index] * self.config.folding_ratio;
            let previous_height = self.hmap[world_index];
            let previous_age = self.amap[world_index];
            self.plates[current].set_crust(x_mod, y_mod, this_height + crust, previous_age);
            self.plates[previous].set_crust(
                x_mod,
                y_mod,
                previous_height * (1.0 - self.config.folding_ratio),
                previous_age,
            );
            self.collisions[previous].push(PlateCollision {
                index: current as u32,
                x: x_mod,
                y: y_mod,
                crust,
            });
            *continental_collisions += 1;
            self.hmap.as_mut_slice()[world_index] = self.plates[current].get_crust(x_mod, y_mod);
            self.imap.as_mut_slice()[world_index] = current as u32;
            self.amap.as_mut_slice()[world_index] =
                self.plates[current].get_crust_timestamp(x_mod, y_mod);
        }

        let _ = local_index;
    }

    fn update_collisions(&mut self) {
        for i in 0..self.plates.len() {
            let collisions = core::mem::take(&mut self.collisions[i]);
            for coll in collisions {
                assert_ne!(i as u32, coll.index);
                let other = coll.index as usize;
                {
                    let (a, b) = two_mut(&mut self.plates, i, other);
                    a.apply_friction(coll.crust);
                    b.apply_friction(coll.crust);
                }
                let (coll_count_i, coll_ratio_i) =
                    self.plates[i].get_collision_info(coll.x, coll.y);
                let (coll_count_j, coll_ratio_j) =
                    self.plates[other].get_collision_info(coll.x, coll.y);
                let coll_count = coll_count_i.min(coll_count_j);
                let coll_ratio = coll_ratio_i.max(coll_ratio_j);
                if coll_count > self.config.aggregation_overlap_abs
                    || coll_ratio > self.config.aggregation_overlap_rel
                {
                    let (donor, receiver) = two_mut(&mut self.plates, i, other);
                    let amount = donor.aggregate_crust(receiver, coll.x, coll.y);
                    receiver.collide(donor, amount);
                }
            }
        }
    }

    fn remove_empty_plates(&mut self) {
        let mut i = 0;
        while i < self.plates.len() {
            if self.plates.len() == 1 {
                break;
            }
            if self.plate_indices_found[i] == 0 {
                let last = self.plates.len() - 1;
                self.plates.swap_remove(i);
                self.plate_indices_found.swap_remove(i);
                self.collisions.swap_remove(i);
                self.subductions.swap_remove(i);
                if i != last {
                    for owner in self.imap.as_mut_slice() {
                        if *owner == last as u32 {
                            *owner = i as u32;
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    fn restart(&mut self) {
        let map_area = self.dimensions.area() as usize;
        self.cycle_count += (self.config.cycle_count > 0) as u32;
        if self.cycle_count > self.config.cycle_count {
            self.plates.clear();
            return;
        }

        self.hmap.as_mut_slice().fill(0.0);
        for plate in &self.plates {
            let x0 = plate.left();
            let y0 = plate.top();
            let x1 = x0 + plate.width();
            let y1 = y0 + plate.height();
            let mut j = 0_usize;
            for y in y0..y1 {
                for x in x0..x1 {
                    let x_mod = self.dimensions.x_mod(x);
                    let y_mod = self.dimensions.y_mod(y);
                    let index = self.dimensions.index_of(x_mod, y_mod);
                    let h0 = self.hmap[index];
                    let h1 = plate.map()[j];
                    let a0 = self.amap[index];
                    let a1 = plate.age_map()[j];
                    let h_sum = h0 + h1;
                    self.amap.as_mut_slice()[index] = if h_sum > 0.0 {
                        ((h0 * a0 as f32 + h1 * a1 as f32) / h_sum) as u32
                    } else {
                        a1
                    };
                    self.hmap.as_mut_slice()[index] += h1;
                    j += 1;
                }
            }
        }

        self.refresh_plate_velocities();
        self.plates.clear();
        if self.cycle_count < self.config.cycle_count + (self.config.cycle_count == 0) as u32 {
            self.create_plates();
            for plate in &mut self.plates {
                let x0 = plate.left();
                let y0 = plate.top();
                let x1 = x0 + plate.width();
                let y1 = y0 + plate.height();
                let mut j = 0_usize;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let x_mod = self.dimensions.x_mod(x);
                        let y_mod = self.dimensions.y_mod(y);
                        plate.age_map_mut()[j] = self.amap[self.dimensions.index_of(x_mod, y_mod)];
                        j += 1;
                    }
                }
            }
            return;
        }

        for i in 0..map_area {
            let mut crust_age = self.iter_count.wrapping_sub(self.amap[i]);
            crust_age = MAX_BUOYANCY_AGE.wrapping_sub(crust_age);
            crust_age &= mask(crust_age <= MAX_BUOYANCY_AGE);
            self.hmap.as_mut_slice()[i] += (self.hmap[i] < CONTINENTAL_BASE) as u32 as f32
                * BUOYANCY_BONUS_X
                * OCEANIC_BASE
                * crust_age as f32
                * MULINV_MAX_BUOYANCY_AGE;
        }
    }

    fn refresh_plate_velocities(&mut self) {
        self.plate_velocities.clear();
        self.plate_velocities
            .extend(self.plates.iter().map(Plate::velocity_vector));
    }
}

fn two_mut<T>(slice: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    assert_ne!(first, second);
    if first < second {
        let (left, right) = slice.split_at_mut(second);
        (&mut left[first], &mut right[0])
    } else {
        let (left, right) = slice.split_at_mut(first);
        (&mut right[0], &mut left[second])
    }
}

fn mask(value: bool) -> u32 {
    if value { u32::MAX } else { 0 }
}

fn create_initial_heightmap(
    dimensions: WorldDimension,
    sea_level: f32,
    random: &mut SimpleRandom,
) -> HeightMap {
    let tmp_dim = WorldDimension::new(dimensions.width() + 1, dimensions.height() + 1);
    let mut tmp = vec![0.0_f32; tmp_dim.area() as usize];
    create_slow_noise(&mut tmp, tmp_dim, *random);

    let mut lowest = tmp[0];
    let mut highest = tmp[0];
    for value in tmp.iter().copied().skip(1) {
        lowest = if lowest < value { lowest } else { value };
        highest = if highest > value { highest } else { value };
    }

    for value in &mut tmp {
        *value = (*value - lowest) / (highest - lowest);
    }

    let mut sea_threshold = 0.5_f32;
    let mut threshold_step = 0.5_f32;
    while threshold_step > 0.01 {
        let mut count = 0_u32;
        for value in tmp.iter().copied() {
            count += (value < sea_threshold) as u32;
        }

        threshold_step *= 0.5;
        if count as f32 / (tmp_dim.area() as f32) < sea_level {
            sea_threshold += threshold_step;
        } else {
            sea_threshold -= threshold_step;
        }
    }

    for value in &mut tmp {
        *value = if *value > sea_threshold {
            *value + CONTINENTAL_BASE
        } else {
            OCEANIC_BASE
        };
    }

    let mut hmap = HeightMap::new(dimensions.width(), dimensions.height(), 0.0);
    for y in 0..dimensions.height() {
        let src = tmp_dim.line_index(y);
        let dst = hmap.line_index(y);
        let width = dimensions.width() as usize;
        hmap.as_mut_slice()[dst..dst + width].copy_from_slice(&tmp[src..src + width]);
    }

    hmap
}

fn create_slow_noise(map: &mut [f32], dimensions: WorldDimension, mut random: SimpleRandom) {
    let seed = random.next_u32() as i64;
    let width = dimensions.width();
    let height = dimensions.height();
    let persistence = 0.25_f32;
    let noise_scale = 0.593_f32;
    let ka = (256_i64 / seed) as f32;
    let kb = (seed * 567 % 256) as f32;
    let seed_mod = seed % 256;
    let kc = (seed_mod * seed_mod % 256) as f32;
    let kd = ((567 - seed) % 256) as f32;

    for y in 0..height {
        for x in 0..width {
            let f_nx = x as f32 / width as f32;
            let f_ny = y as f32 / height as f32;
            let f_rdx = f_nx * 2.0 * core::f32::consts::PI;
            let f_rdy = f_ny * 4.0 * core::f32::consts::PI;
            let a = f_rdx.sin();
            let b = f_rdx.cos();
            let c = f_rdy.sin();
            let d = f_rdy.cos();
            let value = scaled_octave_noise_4d(
                4.0,
                persistence,
                0.25,
                0.0,
                1.0,
                ka + a * noise_scale,
                kb + b * noise_scale,
                kc + c * noise_scale,
                kd + d * noise_scale,
            );
            map[(y * width + x) as usize] = value;
        }
    }
}
