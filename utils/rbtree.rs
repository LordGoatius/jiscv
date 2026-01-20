use core::ptr::NonNull;

#[repr(C)]
enum Color {
    Red   = 0,
    Black = 1,
}

enum Dir {
    Right,
    Left,
}

pub struct RBTree<T: Ord> {
    root: Node<T>
}

pub struct Node<T: Ord> {
    color: Color,
    data: T,
    // Parents always exist
    parent: NonNull<Node<T>>,
    left: Option<NonNull<Node<T>>>,
    right: Option<NonNull<Node<T>>>,
}

impl<T: Ord> Node<T> {
    fn rot_dir(&self) -> Dir {
        if let Some(ptr) = unsafe { (*self).parent.as_ref().right } && ptr == self.into() {
            Dir::Right
        } else {
            Dir::Left
        }
    }
}
