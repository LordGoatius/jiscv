#![rustfmt::skip]

use core::{cmp::Ordering, mem::MaybeUninit, ptr::NonNull};

pub struct BTree<T: Ord, const B: usize> where [(); B+1]: {
    head: BTreeBlock<T, B>,
}

struct BTreeBlock<T: Ord, const B: usize> where [(); B + 1]: {
    height: usize,
    parent: Option<NonNull<BTreeBlock<T, B>>>,
    nodes: [Option<T>; B],
    edges: [MaybeUninit<NonNull<BTreeBlock<T, B>>>; B + 1],
}

impl<T: Ord, const B: usize> BTreeBlock<T, B> where [(); B+1]: {
    pub fn search(&mut self, val: T) -> Option<NonNull<T>> {
        let mut ind = 0;

        // Nodes will always be filled from left to right, so each iteration is a single
        // inc of `ind`
        for i in self.nodes.iter_mut().flatten() {
            match (*i).cmp(&val) {
                Ordering::Less => ind += 1,
                Ordering::Equal => return Some(unsafe { NonNull::new_unchecked(i as *mut T) }),
                Ordering::Greater => break,
            }
        }

        if self.height == 0 {
            return None;
        }

        return unsafe { self.edges[ind].assume_init().as_mut().search(val) };
    }

    pub fn insert(&mut self, val: NonNull<T>) {
        
    }

    pub fn remove(&mut self, val: NonNull<T>) {
        
    }
}
