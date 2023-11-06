use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum GridCoord {
    Valid(i64, i64),
    OutOfBounds,
}

impl GridCoord {
    fn adjust(&self, a: i64, b: i64) -> GridCoord {
        // Implement OutOfBounds

        if let GridCoord::Valid(x, y) = self {
            GridCoord::Valid(x + a, y + b)
        } else {
            GridCoord::OutOfBounds
        }
    }

    fn neighbours(&self) -> [GridCoord; 8] {
        if let GridCoord::Valid(_, _) = self {
            [
                self.adjust(-1, -1),
                self.adjust(0, -1),
                self.adjust(1, -1),
                self.adjust(-1, 0),
                self.adjust(1, 0),
                self.adjust(-1, 1),
                self.adjust(0, 1),
                self.adjust(1, 1),
            ]
        } else {
            [GridCoord::OutOfBounds; 8]
        }
    }
}

struct SparseGrid<T> {
    elements: HashMap<GridCoord, T>,
}

#[allow(unused)]
impl<T: Copy> SparseGrid<T> {
    fn new() -> Self {
        SparseGrid {
            elements: HashMap::new(),
        }
    }

    fn set(&mut self, k: GridCoord, v: T) {
        self.elements.insert(k, v);
    }

    fn unset(&mut self, k: GridCoord) {
        self.elements.remove(&k);
    }

    fn get(&self, k: GridCoord) -> Option<T> {
        match self.elements.get(&k) {
            Some(&v) => Some(v),
            None => None,
        }
    }

    fn count(&self, k: GridCoord) -> usize {
        match k {
            GridCoord::OutOfBounds => 0,
            _ => match self.get(k) {
                Some(_) => 1,
                None => 0,
            },
        }
    }

    fn count_neighbours(&self, k: GridCoord) -> usize {
        let mut count = 0;

        for v in k.neighbours() {
            count += self.count(v);
        }

        count
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
    fn test_get_set_count() {
        let mut g: SparseGrid<usize> = SparseGrid::new();

        assert_eq!(g.get(K1), None);
        assert_eq!(g.count(K1), 0);

        g.set(K1, V);

        assert_eq!(g.get(K1), Some(V));
        assert_eq!(g.get(K2), None);
        assert_eq!(g.count(K1), 1);
        assert_eq!(g.count(K2), 0);

        assert_eq!(g.count(GridCoord::OutOfBounds), 0);
    }

    #[test]
    fn test_neighbours() {
        let mut g: SparseGrid<usize> = SparseGrid::new();

        assert_eq!(g.count_neighbours(K1), 0);

        g.set(K1, V);

        assert_eq!(g.count_neighbours(K1), 0);
        assert_eq!(g.count_neighbours(K2), 1);

        assert_eq!(g.count_neighbours(GridCoord::OutOfBounds), 0);
    }
}
