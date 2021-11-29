use crate::biot::{Biot, TreePoint};
use macroquad::prelude::*;
use rstar::RTree;

/// A collection of biots. Responsible for handling interactions between biots
pub struct BiotCollection {
    biots: Vec<Biot>,
}

impl BiotCollection {
    /// Create n random biots
    pub fn new(n: usize) -> Self {
        let mut s = Self { biots: Vec::new() };
        for _ in 0..n {
            s.biots.push(Biot::random_biot());
        }
        s
    }

    /// Compute one step of the simulation.
    pub fn step(&mut self) {
        let mut new: Vec<Biot> = Vec::new();
        // R-star datastructure used for quickly locating neighbors
        let tree: RTree<TreePoint> = RTree::bulk_load(
            self.biots
                .iter()
                .enumerate()
                .map(|(idx, biot)| TreePoint {
                    idx,
                    x: biot.stats.pos.x as f64,
                    y: biot.stats.pos.y as f64,
                })
                .collect(),
        );
        // Move and reproduce biots
        for n in 0..(self.biots.len()) {
            let mut feed_dir: Option<Vec2> = None;
            if self.biots[n].properties.intelligence > 0. {
                for (other, d2) in tree.nearest_neighbor_iter_with_distance_2(&[
                    self.biots[n].stats.pos.x as f64,
                    self.biots[n].stats.pos.y as f64,
                ]) {
                    if d2 as f32
                        > (self.biots[n].properties.intelligence
                            * self.biots[n].properties.intelligence)
                            * 1600.
                    {
                        break;
                    }
                    if self.biots[n].is_stronger(&self.biots[other.idx]) {
                        // Add small offset to workaround rstart panic. TODO: report it upstream
                        feed_dir = Some(
                            vec2(
                                other.x as f32 - self.biots[n].stats.pos.x + 0.0001,
                                other.y as f32 - self.biots[n].stats.pos.y + 0.0001,
                            )
                            .normalize(),
                        );
                        break;
                    }
                }
            }
            let off = self.biots[n].step(&tree, feed_dir);
            if let Some(offspring) = off {
                new.push(offspring);
            }
        }
        // Compute biot interactions
        for f in tree.iter() {
            for s in tree.locate_within_distance([f.x, f.y], 50.)
            //FIXME 30 is hardcoded
            {
                if f.idx < s.idx {
                    // Don't do it twice
                    Biot::interact(&mut self.biots, f.idx, s.idx);
                }
            }
        }
        // Remove dead biots and add the new ones to the collection
        self.biots.retain(|b| !b.is_dead());
        self.biots.append(&mut new);
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
