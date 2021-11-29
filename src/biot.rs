use core::{ops, slice};
use macroquad::prelude::{rand, screen_height, screen_width, vec2, Vec2};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

/// Genome propeties of biots.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Gene {
    /// A gene that does nothing observable.
    None = 0,
    /// Influences the attack value of the biot.
    Attack = 1,
    /// Influences the defensive value of the biot.
    Defense = 2,
    /// Influences how well the biot can generate energy from sunlight.
    Photosynthesis = 3,
    /// Influences how fast the biot can move around.
    Motion = 4,
    /// Influences the intelligence of the biot.
    Intelligence = 5,
}

impl Gene {
    /// Creates a new random gene.
    pub fn random() -> Self {
        let random = rand::gen_range::<u8>(0, 5);
        match random {
            0 => Self::None,
            1 => Self::Attack,
            2 => Self::Defense,
            3 => Self::Photosynthesis,
            4 => Self::Motion,
            5 => Self::Intelligence,
            _ => unreachable!("encountered unexpected random gene index {random}"),
        }
    }
}

/// The set of genes a biot is made of.
#[derive(Debug, Clone)]
pub struct Genome {
    genes: [Gene; 13],
}

impl Genome {
    /// Creates a random biot genome.
    pub fn random() -> Self {
        let mut genes = [Gene::Attack; 13];
        for gene in &mut genes {
            *gene = Gene::random();
        }
        Self { genes }
    }

    /// Randomly mutate a single gene.
    pub fn mutate(&mut self) {
        let which_gene = rand::gen_range(0, self.genes.len());
        self.genes[which_gene] = Gene::random();
    }

    /// Returns an iterator over the genes of the genome.
    pub fn genes(&self) -> slice::Iter<Gene> {
        self.genes.iter()
    }
}

/// Modulus operator to get toroidal world topology
fn modulus<T>(a: T, b: T) -> T
where
    T: ops::Rem<Output = T> + ops::Add<Output = T> + Copy,
{
    ((a % b) + b) % b
}

/// The properties of a biot.
///
/// The properties are fully derived by the genome of the biot.
#[derive(Debug, Clone, Default)]
pub struct Properties {
    pub attack: f32,
    pub defense: f32,
    pub photosynthesis: f32,
    pub motion: f32,
    pub intelligence: f32,
}

impl Properties {
    /// Reset properties to their default values.
    fn reset(&mut self) {
        self.attack = 0.0;
        self.defense = 0.0;
        self.photosynthesis = 0.0;
        self.motion = 0.0;
        self.intelligence = 0.0;
    }

    /// Compute chacteristics from biot genome
    pub fn adjust_to_genome(&mut self, genome: &Genome) {
        // Reset properties before adjustments:
        self.reset();
        // Recalculate stats from genome:
        for gene in genome.genes() {
            match gene {
                Gene::None => (),
                Gene::Attack => self.attack += 0.1,
                Gene::Defense => self.defense += 0.1,
                Gene::Photosynthesis => self.photosynthesis += 0.1,
                Gene::Motion => self.motion += 0.1,
                Gene::Intelligence => self.intelligence += 10.0,
            }
        }
    }

    /// Calculates the metabolism costs of the properties.
    ///
    /// # Note
    ///
    /// The metabolism indicates how much energy the biot requires for living.
    fn metabolism(&self) -> f32 {
        0.2 * (4.5 * self.attack + 2.3 * self.defense + 2.5 * self.motion + 0.1 * self.intelligence)
    }

    /// Total weight of the biot, useful for computing its motion.
    fn weight(&self) -> f32 {
        self.attack + self.defense + self.photosynthesis + self.motion
    }
}

/// The status values of a biot.
#[derive(Debug, Clone)]
pub struct Stats {
    pub life: f32,
    pub pos: Vec2,
    pub speed: Vec2,
    pub age: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            life: 0.0,
            pos: vec2(0.0, 0.0),
            speed: vec2(0.0, 0.0),
            age: 0,
        }
    }
}

impl Stats {
    /// Positions the biot randomly on the screen.
    pub fn position_randomly(&mut self) {
        self.pos.x = rand::gen_range(0., 1.) * screen_width();
        self.pos.y = rand::gen_range(0., 1.) * screen_height();
    }
}

/// A biot.
#[derive(Clone, Debug)]
pub struct Biot {
    pub stats: Stats,
    genome: Genome,
    pub properties: Properties,
}

impl Biot {
    /// Creates a random biot.
    pub fn random_biot() -> Self {
        let genome = Genome::random();
        let mut properties = Properties::default();
        properties.adjust_to_genome(&genome);
        let mut stats = Stats::default();
        stats.position_randomly();
        let mut s = Self {
            stats,
            genome,
            properties,
        };
        s.stats.life = s.base_life();
        s
    }

    /// Compute the evolution of the biot for one simulation step
    pub fn step(&mut self, rtree: &RTree<TreePoint>, feed_dir: Option<Vec2>) -> Option<Biot> {
        let mut offspring = None;
        let adult_factor = 4.;
        if self.stats.life >= self.base_life() * adult_factor {
            let close_by = rtree
                .nearest_neighbor_iter_with_distance_2(&[
                    self.stats.pos.x as f64,
                    self.stats.pos.y as f64,
                ])
                .nth(5);
            if close_by.map_or(true, |(_, d2)| d2 > 200.) {
                let mut off = self.clone();
                off.stats.age = 0;
                while rand::gen_range(0., 1.) < 0.2 {
                    off.mutate();
                }
                off.stats.life = off.base_life();
                off.random_move(1.5);
                offspring = Some(off);
                self.stats.life = (adult_factor - 1.0) * self.base_life();
            }
        }
        self.stats.pos += self.stats.speed;
        self.stats.pos.x = modulus(self.stats.pos.x, screen_width());
        self.stats.pos.y = modulus(self.stats.pos.y, screen_height());
        self.stats.speed *= 0.9;
        self.stats.life += (self.properties.photosynthesis - self.properties.metabolism()) * 0.4;
        if rand::gen_range(0., 1.) < 0.2 * self.properties.motion {
            let speed = 7. * self.properties.motion / self.properties.weight();
            if self.properties.intelligence > 0. {
                if let Some(feed_dir) = feed_dir {
                    self.accelerate(feed_dir, speed);
                } else {
                    self.random_move(speed)
                }
            } else {
                self.random_move(speed)
            }
        }
        self.stats.age += 1;
        offspring
    }

    /// Compute the interaction between two biots.
    pub fn interact(biots: &mut Vec<Self>, i: usize, j: usize) {
        let dist = (biots[i].stats.pos - biots[j].stats.pos).length();
        if dist < 10.0 * (biots[i].properties.weight() + biots[j].properties.weight()) {
            if biots[i].is_stronger(&biots[j]) {
                biots[i].stats.life += biots[j].stats.life * 0.8;
                biots[j].stats.life = 0.0;
            } else if biots[j].is_stronger(&biots[i]) {
                biots[j].stats.life += biots[i].stats.life * 0.8;
                biots[i].stats.life = 0.0;
            }
        }
    }

    /// Returns `true` if the biot is dead.
    pub fn is_dead(&self) -> bool {
        self.stats.life <= 0.0 || self.stats.age >= 10000
    }

    /// Returns `true` if `self` is stronger than `other`.
    pub fn is_stronger(&self, other: &Self) -> bool {
        self.properties.attack > other.properties.attack + other.properties.defense * 0.8
    }

    /// Move the biot in a random direction.
    fn random_move(&mut self, speed: f32) {
        self.accelerate(
            vec2(
                rand::gen_range(0.0, 1.0) - 0.5,
                rand::gen_range(0.0, 1.0) - 0.5,
            )
            .normalize(),
            speed,
        );
    }

    /// Apply acceleration in a certain direction.
    fn accelerate(&mut self, dir: Vec2, speed: f32) {
        self.stats.speed += dir * speed;
    }

    /// Randomly mutates a single gene in the genome of the biot.
    fn mutate(&mut self) {
        self.genome.mutate();
        self.properties.adjust_to_genome(&self.genome);
    }

    /// Original life points of a biot.
    ///
    /// # Note
    ///
    /// This is also used to determine when the biot will spawn.
    fn base_life(&self) -> f32 {
        8.0 * self.properties.weight()
    }
}

/// Helper structure used for the rstar geometric data structure. This data structure is used for
/// computing interaction between biots fluidly even with thousands of them
pub struct TreePoint {
    pub x: f64,
    pub y: f64,
    pub idx: usize,
}

impl RTreeObject for TreePoint {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for TreePoint {
    fn distance_2(
        &self,
        point: &<<Self as rstar::RTreeObject>::Envelope as rstar::Envelope>::Point,
    ) -> <<<Self as rstar::RTreeObject>::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar
    {
        (self.x - point[0]) * (self.x - point[0]) + (self.y - point[1]) * (self.y - point[1])
    }
}
