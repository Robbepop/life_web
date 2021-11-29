use crate::biot::{Biot, TreePoint};
use macroquad::prelude::*;
use rstar::RTree;

/// A collection of biots. Responsible for handling interactions between biots
pub struct BiotCollection {
    biots: Vec<Biot>,
    offsprings: Vec<Biot>,
}

impl BiotCollection {
    /// Create `len` random biots.
    pub fn new(len: usize) -> Self {
        let biots = (0..len).map(|_| Biot::random_biot()).collect::<Vec<_>>();
        let offsprings = Vec::new();
        Self { biots, offsprings }
    }

    /// Compute one step of the simulation.
    pub fn step(&mut self) {
        // Clear offsprings in case there are still some from last step.
        self.offsprings.clear();
        // R-star datastructure used for quickly locating neighbors.
        let tree: RTree<TreePoint> = RTree::bulk_load(
            self.biots
                .iter()
                .enumerate()
                .map(|(idx, biot)| TreePoint {
                    idx,
                    x: biot.stats.pos.x,
                    y: biot.stats.pos.y,
                })
                .collect(),
        );
        // Move and reproduce biots.
        for idx in 0..(self.biots.len()) {
            let mut feed_dir: Option<Vec2> = None;
            let intelligence = self.biots[idx].properties.intelligence;
            if intelligence > 0.0 {
                let pos = [
                    self.biots[idx].stats.pos.x,
                    self.biots[idx].stats.pos.y,
                ];
                for (neighbour, squared_distance) in
                    tree.nearest_neighbor_iter_with_distance_2(&pos)
                {
                    if idx == neighbour.idx {
                        // Do not move towards itself or produce with itself.
                        continue;
                    }
                    let max_detection_distance = (intelligence * intelligence) * 1600.0;
                    if squared_distance as f32 > max_detection_distance {
                        // Victim is out of reach.
                        //
                        // Further iterated elements are farther away so we can break here.
                        break;
                    }
                    if self.biots[idx].is_stronger(&self.biots[neighbour.idx]) {
                        feed_dir = Some(
                            vec2(
                                neighbour.x as f32 - self.biots[idx].stats.pos.x,
                                neighbour.y as f32 - self.biots[idx].stats.pos.y,
                            )
                            .normalize(),
                        );
                        break;
                    }
                }
            }
            let off = self.biots[idx].step(&tree, feed_dir);
            if let Some(offspring) = off {
                self.offsprings.push(offspring);
            }
        }
        // Compute biot interactions.
        for f in &tree {
            for s in tree.locate_within_distance([f.x, f.y], 50.0)
            //FIXME 30 is hardcoded
            {
                if f.idx < s.idx {
                    // Don't do it twice
                    Biot::interact(&mut self.biots, f.idx, s.idx);
                }
            }
        }
        // Remove dead biots and append the offsprings to the collection.
        self.biots.retain(Biot::is_alive);
        self.biots.append(&mut self.offsprings);
    }

    /// Display the biot collection
    pub fn draw(&self) {
        for biot in self.biots.iter() {
            if biot.properties.intelligence > 0. {
                let size = 14.
                    * (biot.properties.photosynthesis
                        + biot.properties.attack
                        + biot.properties.defense
                        + biot.properties.motion);
                draw_rectangle(
                    biot.stats.pos.x - size / 2.,
                    biot.stats.pos.y - size / 2.,
                    size,
                    size,
                    GREEN,
                );
            }
            draw_circle(
                biot.stats.pos.x,
                biot.stats.pos.y,
                7. * (biot.properties.photosynthesis
                    + biot.properties.attack
                    + biot.properties.defense
                    + biot.properties.motion),
                GREEN,
            );
            draw_circle(
                biot.stats.pos.x,
                biot.stats.pos.y,
                7. * (biot.properties.attack + biot.properties.defense + biot.properties.motion),
                RED,
            );
            draw_circle(
                biot.stats.pos.x,
                biot.stats.pos.y,
                7. * (biot.properties.defense + biot.properties.motion),
                DARKBLUE,
            );
            draw_circle(
                biot.stats.pos.x,
                biot.stats.pos.y,
                7. * (biot.properties.motion),
                BLUE,
            );
        }
    }

    /// The number of biots currently in our collection
    pub fn len(&self) -> usize {
        self.biots.len()
    }
}
