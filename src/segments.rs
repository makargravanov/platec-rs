// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::bounds::Bounds;
use super::geometry::WorldDimension;
use super::rectangle::Rectangle;
use super::segment_data::SegmentData;

pub type ContinentId = u32;
pub const CONT_BASE: f32 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Segments {
    ids: Vec<ContinentId>,
    data: Vec<SegmentData>,
}

impl Segments {
    pub fn new(area: u32) -> Self {
        Self {
            ids: vec![u32::MAX; area as usize],
            data: Vec::new(),
        }
    }

    pub fn area(&self) -> u32 {
        self.ids.len() as u32
    }
    pub fn reset(&mut self) {
        self.ids.fill(u32::MAX);
        self.data.clear();
    }
    pub fn reassign(&mut self, ids: Vec<ContinentId>) {
        self.ids = ids;
    }
    pub fn shift(&mut self, dx: u32, dy: u32) {
        for data in &mut self.data {
            data.shift(dx, dy);
        }
    }
    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }
    pub fn id(&self, index: usize) -> ContinentId {
        self.ids[index]
    }
    pub fn set_id(&mut self, index: usize, id: ContinentId) {
        self.ids[index] = id;
    }
    pub fn ids(&self) -> &[ContinentId] {
        &self.ids
    }
    pub fn data(&self, id: ContinentId) -> &SegmentData {
        &self.data[id as usize]
    }
    pub fn data_mut(&mut self, id: ContinentId) -> &mut SegmentData {
        &mut self.data[id as usize]
    }

    pub fn get_continent_at(
        &mut self,
        x: u32,
        y: u32,
        bounds: &Bounds,
        map: &[f32],
        dimensions: WorldDimension,
    ) -> ContinentId {
        let mut lx = x;
        let mut ly = y;
        let index = bounds.get_valid_map_index(&mut lx, &mut ly) as usize;
        let mut segment = self.id(index);
        if segment >= self.size() {
            segment = self.create_segment(lx, ly, bounds, map, dimensions);
        }
        assert!(segment < self.size());
        segment
    }

    fn create_segment(
        &mut self,
        x: u32,
        y: u32,
        bounds: &Bounds,
        map: &[f32],
        dimensions: WorldDimension,
    ) -> ContinentId {
        let bounds_width = bounds.width();
        let bounds_height = bounds.height();
        let origin_index = bounds.index(x, y) as usize;
        let id = self.size();

        if self.id(origin_index) < id {
            return self.id(origin_index);
        }

        let neighbor_id = self.calc_direction(x, y, origin_index, id, bounds, map);
        if neighbor_id < id {
            self.set_id(origin_index, neighbor_id);
            let data = self.data_mut(neighbor_id);
            data.inc_area();
            data.enlarge_to_contain(x, y);
            return neighbor_id;
        }

        let mut data = SegmentData::new(Rectangle::new(dimensions, x, x, y, y), 0);
        let mut spans_todo = vec![Vec::<u32>::new(); bounds_height as usize];
        let mut spans_done = vec![Vec::<u32>::new(); bounds_height as usize];
        self.set_id(origin_index, id);
        spans_todo[y as usize].push(x);
        spans_todo[y as usize].push(x);

        loop {
            let mut lines_processed = 0_u32;
            for line in 0..bounds_height {
                if spans_todo[line as usize].is_empty() {
                    continue;
                }
                let Some((mut start, mut end)) =
                    scan_spans(line, bounds_width, &mut spans_todo, &spans_done)
                else {
                    continue;
                };
                if start > end {
                    continue;
                }

                let row_above = if line > 0 {
                    line - 1
                } else {
                    bounds_height - 1
                };
                let row_below = if line < bounds_height - 1 {
                    line + 1
                } else {
                    0
                };
                let line_here = line * bounds_width;
                let line_above = row_above * bounds_width;
                let line_below = row_below * bounds_width;

                while start > 0
                    && self.id((line_here + start - 1) as usize) > id
                    && map[(line_here + start - 1) as usize] >= CONT_BASE
                {
                    start -= 1;
                    self.set_id((line_here + start) as usize, id);
                }
                while end < bounds_width - 1
                    && self.id((line_here + end + 1) as usize) > id
                    && map[(line_here + end + 1) as usize] >= CONT_BASE
                {
                    end += 1;
                    self.set_id((line_here + end) as usize, id);
                }

                if bounds_width == dimensions.width()
                    && start == 0
                    && self.id((line_here + bounds_width - 1) as usize) > id
                    && map[(line_here + bounds_width - 1) as usize] >= CONT_BASE
                {
                    self.set_id((line_here + bounds_width - 1) as usize, id);
                    spans_todo[line as usize].push(bounds_width - 1);
                    spans_todo[line as usize].push(bounds_width - 1);
                }
                if bounds_width == dimensions.width()
                    && end == bounds_width - 1
                    && self.id(line_here as usize) > id
                    && map[line_here as usize] >= CONT_BASE
                {
                    self.set_id(line_here as usize, id);
                    spans_todo[line as usize].push(0);
                    spans_todo[line as usize].push(0);
                }

                data.inc_area_by(1 + end - start);
                if line < data.top() {
                    data.set_top(line);
                }
                if line > data.bottom() {
                    data.set_bottom(line);
                }
                if start < data.left() {
                    data.set_left(start);
                }
                if end > data.right() {
                    data.set_right(end);
                }

                if line > 0 || bounds_height == dimensions.height() {
                    queue_neighbor_spans(
                        self,
                        map,
                        id,
                        bounds_width,
                        start,
                        end,
                        line_above,
                        row_above,
                        &mut spans_todo,
                    );
                }
                if line < bounds_height - 1 || bounds_height == dimensions.height() {
                    queue_neighbor_spans(
                        self,
                        map,
                        id,
                        bounds_width,
                        start,
                        end,
                        line_below,
                        row_below,
                        &mut spans_todo,
                    );
                }

                spans_done[line as usize].push(start);
                spans_done[line as usize].push(end);
                lines_processed += 1;
            }
            if lines_processed == 0 {
                break;
            }
        }

        self.data.push(data);
        id
    }

    fn calc_direction(
        &self,
        x: u32,
        y: u32,
        origin_index: usize,
        id: ContinentId,
        bounds: &Bounds,
        map: &[f32],
    ) -> ContinentId {
        let width = bounds.width() as usize;
        let can_go_left = x > 0 && map[origin_index - 1] >= CONT_BASE;
        let can_go_right = x < bounds.width() - 1 && map[origin_index + 1] >= CONT_BASE;
        let can_go_up = y > 0 && map[origin_index - width] >= CONT_BASE;
        let can_go_down = y < bounds.height() - 1 && map[origin_index + width] >= CONT_BASE;

        if can_go_left && self.id(origin_index - 1) < id {
            self.id(origin_index - 1)
        } else if can_go_right && self.id(origin_index + 1) < id {
            self.id(origin_index + 1)
        } else if can_go_up && self.id(origin_index - width) < id {
            self.id(origin_index - width)
        } else if can_go_down && self.id(origin_index + width) < id {
            self.id(origin_index + width)
        } else {
            id
        }
    }
}

fn scan_spans(
    line: u32,
    bounds_width: u32,
    spans_todo: &mut [Vec<u32>],
    spans_done: &[Vec<u32>],
) -> Option<(u32, u32)> {
    let line_index = line as usize;
    while !spans_todo[line_index].is_empty() {
        let mut end = spans_todo[line_index].pop().unwrap();
        let mut start = spans_todo[line_index].pop().unwrap();
        for done in spans_done[line_index].chunks_exact(2) {
            if start >= done[0] && start <= done[1] {
                start = done[1] + 1;
            }
            if end >= done[0] && end <= done[1] {
                end = done[0].wrapping_sub(1);
            }
        }
        if end >= bounds_width {
            start = u32::MAX;
            end -= 1;
        }
        if start <= end {
            return Some((start, end));
        }
    }
    None
}

fn queue_neighbor_spans(
    segments: &mut Segments,
    map: &[f32],
    id: ContinentId,
    bounds_width: u32,
    start: u32,
    end: u32,
    line_offset: u32,
    row: u32,
    spans_todo: &mut [Vec<u32>],
) {
    let mut j = start;
    while j <= end {
        let index = (line_offset + j) as usize;
        if segments.id(index) > id && map[index] >= CONT_BASE {
            let a = j;
            segments.set_id(index, id);
            j += 1;
            while j < bounds_width {
                let next = (line_offset + j) as usize;
                if !(segments.id(next) > id && map[next] >= CONT_BASE) {
                    break;
                }
                segments.set_id(next, id);
                j += 1;
            }
            let b = j - 1;
            spans_todo[row as usize].push(a);
            spans_todo[row as usize].push(b);
        }
        if j == u32::MAX {
            break;
        }
        j += 1;
    }
}
