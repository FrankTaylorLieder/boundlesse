use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum GridCoord {
    Valid(i64, i64),
    OutOfBounds,
}

impl GridCoord {
    pub fn adjust(&self, a: i64, b: i64) -> GridCoord {
        // TODO Implement OutOfBounds

        if let GridCoord::Valid(x, y) = self {
            GridCoord::Valid(x + a, y + b)
        } else {
            GridCoord::OutOfBounds
        }
    }

    pub fn expand(&self) -> [GridCoord; 9] {
        if let GridCoord::Valid(_, _) = self {
            [
                self.adjust(-1, -1),
                self.adjust(0, -1),
                self.adjust(1, -1),
                self.adjust(-1, 0),
                self.clone(),
                self.adjust(1, 0),
                self.adjust(-1, 1),
                self.adjust(0, 1),
                self.adjust(1, 1),
            ]
        } else {
            [GridCoord::OutOfBounds; 9]
        }
    }
}

pub struct SparseGridOld {
    elements: HashMap<GridCoord, usize>,
}

#[allow(unused)]
impl SparseGridOld {
    pub fn new() -> Self {
        SparseGridOld {
            elements: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: GridCoord, v: usize) {
        self.elements.insert(k, v);
    }

    pub fn unset(&mut self, k: GridCoord) {
        self.elements.remove(&k);
    }

    pub fn get(&self, k: &GridCoord) -> Option<usize> {
        match self.elements.get(k) {
            Some(&v) => Some(v),
            None => None,
        }
    }

    pub fn is_alive(&self, k: &GridCoord) -> bool {
        self.get(k).is_some()
    }

    pub fn elements(&self) -> Vec<GridCoord> {
        self.elements.keys().map(|k| *k).collect()
    }

    fn expand(&self) -> Vec<GridCoord> {
        let iter = self.elements.iter();
        let mut elements: Vec<GridCoord> = Vec::with_capacity(iter.len());
        for (&gc, _) in iter {
            elements.push(gc);
        }

        elements
    }

    fn tally(&mut self, cells: &[GridCoord]) {
        for c in cells {
            let count = match self.elements.get(&c) {
                Some(&v) => v + 1,
                None => 1,
            };

            self.set(*c, count);
        }
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&GridCoord, &mut usize) -> bool,
    {
        self.elements.retain(f);
    }

    pub fn len(&self) -> usize {
        self.elements.keys().len()
    }
}

pub struct UniverseOld {
    pub grid: SparseGridOld,
    pub generation: usize,
}

#[allow(unused)]
impl UniverseOld {
    pub fn new() -> UniverseOld {
        UniverseOld {
            grid: SparseGridOld::new(),
            generation: 0,
        }
    }

    pub fn update(&mut self) -> usize {
        self.generation += 1;
        let mut cell_count: usize = 0;
        let mut next = SparseGridOld::new();
        for c in self.grid.elements() {
            next.tally(&c.expand());
            cell_count += 1;
        }

        next.retain(|gc, v| *v == 3 || (*v == 4 && self.grid.is_alive(gc)));

        self.grid = next;

        cell_count
    }
}

#[derive(Debug)]
pub struct Cell {
    pub is_alive: bool,
    pub generation: usize,
    pub tally: usize,
}

#[derive(Debug)]
pub struct SparseGridGenerations {
    pub elements: HashMap<GridCoord, Cell>,
    pub generation: usize,
}

#[allow(unused)]
impl SparseGridGenerations {
    pub fn new() -> Self {
        SparseGridGenerations {
            elements: HashMap::new(),
            generation: 0,
        }
    }

    pub fn set(&mut self, k: GridCoord) {
        self.elements.insert(
            k,
            Cell {
                is_alive: true,
                generation: self.generation,
                tally: 0,
            },
        );
    }

    pub fn unset(&mut self, k: GridCoord) {
        // TODO Do we need to optimise this too to remember recently removed cells?
        self.elements.remove(&k);
    }

    pub fn is_alive(&self, k: &GridCoord) -> bool {
        match self.elements.get(k) {
            Some(v) => v.is_alive,
            None => false,
        }
    }

    pub fn live_cells(&self) -> Vec<GridCoord> {
        self.elements
            .iter()
            .filter(|&(k, v)| v.is_alive)
            .map(|(k, _)| *k)
            .collect()
    }

    pub fn live_cells_ref(&self) -> Vec<&GridCoord> {
        self.elements
            .iter()
            .filter(|&(k, v)| v.is_alive)
            .map(|(k, _)| k)
            .collect()
    }

    // Tally for gen+1
    fn tally(&mut self, generation: usize, cells: &[GridCoord]) {
        for c in cells {
            match self.elements.get_mut(&c) {
                Some(v) => {
                    if v.generation < generation {
                        v.generation = generation;
                        v.tally = 1;
                    } else {
                        v.tally += 1;
                    }
                }
                None => {
                    self.elements.insert(
                        *c,
                        Cell {
                            is_alive: false,
                            generation,
                            tally: 1,
                        },
                    );
                }
            };
        }
    }

    // Complete the generation...
    fn finalise(&mut self, generation: usize) {
        self.elements.retain(|k, v| {
            // Firstly... adjust the cells to the correct life
            // if v.generation < generation {
            //     // This cell has no neighbours... so will die
            //     v.is_alive = false;
            // } else {
            if v.generation == generation {
                // This cell has some neighbours, so might live.
                let t = v.tally;
                v.is_alive = t == 3 || (t == 4 && v.is_alive);
            }

            //println!("Finalise: {:?} => {:?}", k, v);

            // Discard cells which had no neighbours
            v.generation == generation
        });
    }

    // pub fn len(&self) -> usize {
    //     self.elements.keys().len()
    // }
}

pub struct Universe {
    pub grid: SparseGridGenerations,
    pub generation: usize,
}

#[allow(unused)]
impl Universe {
    pub fn new() -> Self {
        Universe {
            grid: SparseGridGenerations::new(),
            generation: 0,
        }
    }

    pub fn update(&mut self) -> usize {
        self.generation += 1;
        let mut cell_count: usize = 0;
        for c in self.grid.live_cells() {
            //println!("Cell: {:?}", c);
            self.grid.tally(self.generation, &c.expand());
            cell_count += 1;
        }

        // println!("Tallied: {:?}", self.grid);

        self.grid.finalise(self.generation);

        cell_count
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    const V: usize = 42;
    const K1: GridCoord = GridCoord::Valid(0, 0);
    const K2: GridCoord = GridCoord::Valid(0, 1);
    const K3: GridCoord = GridCoord::Valid(0, 2);

    const K4: GridCoord = GridCoord::Valid(-1, 1);
    const K5: GridCoord = GridCoord::Valid(1, 1);

    #[test]
    fn test_adjust() {
        let k1 = K1.adjust(0, 0);
        assert_eq!(k1, K1);

        let k2 = K1.adjust(0, 1);
        assert_eq!(k2, K2);

        let k3 = K1.adjust(-1, -1);
        assert_eq!(k3, GridCoord::Valid(-1, -1));

        // TODO rest of the cases
    }

    #[test]
    fn test_get_set_orig() {
        let mut g: SparseGridOld = SparseGridOld::new();

        assert_eq!(g.get(&K1), None);

        g.set(K1, V);

        assert_eq!(g.get(&K1), Some(V));
        assert_eq!(g.get(&K2), None);
    }

    #[test]
    fn test_get_set_generations() {
        let mut g: SparseGridGenerations = SparseGridGenerations::new();

        assert_eq!(g.is_alive(&K1), false);

        g.set(K1);

        assert_eq!(g.is_alive(&K1), true);
        assert_eq!(g.is_alive(&K2), false);
    }

    #[test]
    fn test_retain_orig() {
        let mut g = SparseGridOld::new();

        g.set(K1, 1);
        g.set(K2, 2);

        assert_eq!(g.get(&K1), Some(1));
        assert_eq!(g.get(&K2), Some(2));

        g.retain(|_k, v| *v > 1);

        assert_eq!(g.get(&K1), None);
        assert_eq!(g.get(&K2), Some(2));
    }

    #[test]
    fn test_tally_orig() {
        let mut g = SparseGridOld::new();

        g.tally(&[K1, K2]);
        g.tally(&[K2]);

        assert_eq!(g.get(&K1), Some(1));
        assert_eq!(g.get(&K2), Some(2));
    }

    #[test]
    fn test_blinker_generations() {
        let mut universe = Universe::new();

        universe.grid.set(K1);
        universe.grid.set(K2);
        universe.grid.set(K3);

        println!("Grid: {:?}", universe.grid);

        let c = universe.update();

        println!("Update: {:?}", universe.grid);

        assert_eq!(c, 3);

        assert_eq!(universe.grid.is_alive(&K1), false);
        assert_eq!(universe.grid.is_alive(&K2), true);
        assert_eq!(universe.grid.is_alive(&K3), false);
        assert_eq!(universe.grid.is_alive(&K4), true);
        assert_eq!(universe.grid.is_alive(&K5), true);
    }
}
