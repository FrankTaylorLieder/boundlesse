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

pub struct SparseGrid {
    elements: HashMap<GridCoord, usize>,
}

#[allow(unused)]
impl SparseGrid {
    pub fn new() -> Self {
        SparseGrid {
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

pub struct Universe {
    pub grid: SparseGrid,
    pub generation: usize,
}

#[allow(unused)]
impl Universe {
    pub fn new() -> Universe {
        Universe {
            grid: SparseGrid::new(),
            generation: 0,
        }
    }

    pub fn update(&mut self) -> usize {
        self.generation += 1;
        let mut cell_count: usize = 0;
        let mut next = SparseGrid::new();
        for c in self.grid.elements() {
            next.tally(&c.expand());
            cell_count += 1;
        }

        next.retain(|gc, v| *v == 3 || (*v == 4 && self.grid.is_alive(gc)));

        self.grid = next;

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
    fn test_get_set() {
        let mut g: SparseGrid = SparseGrid::new();

        assert_eq!(g.get(&K1), None);

        g.set(K1, V);

        assert_eq!(g.get(&K1), Some(V));
        assert_eq!(g.get(&K2), None);
    }

    #[test]
    fn test_retain() {
        let mut g = SparseGrid::new();

        g.set(K1, 1);
        g.set(K2, 2);

        assert_eq!(g.get(&K1), Some(1));
        assert_eq!(g.get(&K2), Some(2));

        g.retain(|_k, v| *v > 1);

        assert_eq!(g.get(&K1), None);
        assert_eq!(g.get(&K2), Some(2));
    }

    #[test]
    fn test_tally() {
        let mut g = SparseGrid::new();

        g.tally(&[K1, K2]);
        g.tally(&[K2]);

        assert_eq!(g.get(&K1), Some(1));
        assert_eq!(g.get(&K2), Some(2));
    }
}
