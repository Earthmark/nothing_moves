use std::{
    cell::RefCell,
    collections::{BinaryHeap, HashMap, HashSet},
    rc::{Rc, Weak},
};

pub struct Maze<const DIMS: usize> {
    walks: HashSet<([u8; DIMS], [u8; DIMS])>,
    lengths: [u8; DIMS],
}

impl<const DIMS: usize> Default for Maze<DIMS> {
    fn default() -> Self {
        Self {
            walks: Default::default(),
            lengths: [1; DIMS],
        }
    }
}

impl<const DIMS: usize> Maze<DIMS> {
    // Generate a maze with the provided number of side lengths.
    pub fn new(lengths: &[u8; DIMS], rng: &mut impl rand::Rng) -> Maze<DIMS> {
        let cell_count = lengths.iter().map(|f| *f as usize).product();

        // Indexed by dimension sums (higher is higher power).
        let mut cells = HashMap::<[u8; DIMS], MazeGenCellRef>::with_capacity(cell_count);
        for index in 0..cell_count {
            let pos = unwrap_index(lengths, index).unwrap();
            cells.insert(pos, MazeGenCell::new(index));
        }

        let mut pending_edges = BinaryHeap::with_capacity(cell_count * DIMS);
        for index in 0..cell_count {
            for dim in 0..DIMS {
                pending_edges.push((rng.next_u32(), index, dim))
            }
        }

        // In general, each cell will be linked with at most one other, but this will be less.
        let mut walks = HashSet::with_capacity(cell_count);

        while let Some((_, target_index, dim)) = pending_edges.pop() {
            let a = unwrap_index(lengths, target_index).unwrap();
            // Skip the ends of each dimension, as that's checking outside the bounds of the space.
            // In the future do this check on insertion into the heap.
            if a[dim] == lengths[dim] {
                continue;
            }
            let mut b = a;
            b[dim] += 1;
            if let Some(cell_a) = cells.get(&a) {
                if let Some(cell_b) = cells.get(&b) {
                    if MazeGenCell::try_merge(cell_a, cell_b) {
                        walks.insert((a, b));
                    }
                }
            }
        }

        walks.shrink_to_fit();

        Maze::<DIMS> {
            lengths: *lengths,
            walks,
        }
    }

    fn check_pair(&self, a: &[u8; DIMS], b: &[u8; DIMS]) -> Option<bool> {
        for index in 0..DIMS {
            let length = self.lengths[index];
            if a[index] >= length || b[index] >= length {
                return None;
            }
        }

        // Check for either direction because it's cheaper to check twice
        // than store an exponential memory problem.
        Some(self.walks.contains(&(*a, *b)) || self.walks.contains(&(*b, *a)))
    }

    pub fn can_move(&self, point: &[u8; DIMS], dimension: usize) -> Option<bool> {
        let mut target_point = *point;
        if let Some(shift_axis) = target_point.get_mut(dimension) {
            if let Some(new_shifted) = shift_axis.checked_add(1) {
                *shift_axis = new_shifted;
                return self.check_pair(point, &target_point);
            }
        }
        None
    }

    #[inline]
    pub fn lengths(&self) -> &[u8; DIMS] {
        &self.lengths
    }
}

struct MazeGenCell {
    id: usize,
    parent: Weak<RefCell<Self>>,
}

type MazeGenCellRef = Rc<RefCell<MazeGenCell>>;

impl MazeGenCell {
    fn new(id: usize) -> MazeGenCellRef {
        Rc::new(RefCell::new(MazeGenCell {
            id,
            parent: Weak::default(),
        }))
    }

    /// Gets the root of the particular cell tree.
    fn get_root(s: &MazeGenCellRef) -> MazeGenCellRef {
        let rc = s.as_ref().borrow();
        if let Some(p) = rc.parent.upgrade() {
            Self::get_root(&p)
        } else {
            s.clone()
        }
    }
    /// Attempts to merge both cells, returning true if they were different trees previously.
    fn try_merge(a: &MazeGenCellRef, b: &MazeGenCellRef) -> bool {
        let ra = Self::get_root(a);
        let rb = Self::get_root(b);
        let pa = ra.borrow();
        if pa.id != rb.borrow().id {
            rb.borrow_mut().parent = Rc::downgrade(&ra);
            true
        } else {
            false
        }
    }
}

fn unwrap_index<const DIMS: usize>(lengths: &[u8; DIMS], index: usize) -> Option<[u8; DIMS]> {
    let mut result = [0; DIMS];
    let mut remaining_index = index;
    for (length, res) in lengths.iter().zip(result.iter_mut()) {
        *res = (remaining_index % (*length as usize)) as u8;
        remaining_index /= *length as usize;
    }
    if remaining_index == 0 {
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn maze_cell_ref_merge_roots() {
        let c0 = MazeGenCell::new(0);
        let c1 = MazeGenCell::new(1);
        let c2 = MazeGenCell::new(2);

        assert_eq!(MazeGenCell::try_merge(&c0, &c1), true);
        assert_eq!(MazeGenCell::try_merge(&c0, &c1), false);
        assert_eq!(MazeGenCell::try_merge(&c1, &c0), false);

        assert_eq!(MazeGenCell::try_merge(&c1, &c2), true);
        assert_eq!(MazeGenCell::try_merge(&c0, &c2), false);
    }

    #[test]
    fn maze_cell_ref_merge_roots_alternate() {
        let c0 = MazeGenCell::new(0);
        let c1 = MazeGenCell::new(1);
        let c2 = MazeGenCell::new(2);

        assert_eq!(MazeGenCell::try_merge(&c0, &c1), true);
        assert_eq!(MazeGenCell::try_merge(&c0, &c1), false);
        assert_eq!(MazeGenCell::try_merge(&c1, &c0), false);

        assert_eq!(MazeGenCell::try_merge(&c0, &c2), true);
        assert_eq!(MazeGenCell::try_merge(&c1, &c2), false);
    }

    #[test]
    fn unwrap_index_verify() {
        assert_eq!(unwrap_index(&[2], 0), Some([0]));
        assert_eq!(unwrap_index(&[2], 1), Some([1]));
        assert_eq!(unwrap_index(&[2], 2), None);
    }

    #[test]
    fn verify_generates() {
        let mut rng = StdRng::seed_from_u64(684153987);
        let maze = Maze::new(&[5, 5, 5, 5, 5], &mut rng);

        assert_eq!(maze.can_move(&[1, 2, 52, 2, 2], 2), None);
    }

    #[test]
    fn verify_generates_single() {
        let mut rng = StdRng::seed_from_u64(684153987);
        let maze = Maze::new(&[5, 1, 1], &mut rng);

        assert_eq!(maze.can_move(&[0, 0, 0], 0), Some(true));
        assert_eq!(maze.can_move(&[1, 0, 0], 0), Some(true));
        assert_eq!(maze.can_move(&[2, 0, 0], 0), Some(true));
        assert_eq!(maze.can_move(&[3, 0, 0], 0), Some(true));
        assert_eq!(maze.can_move(&[4, 0, 0], 0), None);
    }
}
