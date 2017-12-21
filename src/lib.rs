//! This crate helps test the following drop issues:
//!
//! - Assert that an item was dropped
//! - Assert that an item was not dropped
//! - Assert that an item was not dropped multiple times (this is implicit tested)
//!
//! This kind of test is useful for objects that manages the lifetime of other objects, like smart
//! pointers and containers.
//!
//! # Examples
//!
//! Test if [`std::mem::forget`](https://doc.rust-lang.org/stable/std/mem/fn.forget.html) works.
//!
//! ```
//! use testdrop::TestDrop;
//! use std::mem;
//!
//! let td = TestDrop::new();
//! let (id, item) = td.new_item();
//!
//! mem::forget(item);
//!
//! td.assert_no_drop(id);
//! ```
//!
//! Test if the [`std::rc::Rc`](https://doc.rust-lang.org/stable/std/rc/struct.Rc.html) drop
//! implementation works.
//!
//! ```
//! use testdrop::TestDrop;
//! use std::rc::Rc;
//!
//! let td = TestDrop::new();
//! let (id, item) = td.new_item();
//! let item = Rc::new(item);
//! let item_clone = item.clone();
//!
//! // Decrease the reference counter, but do not drop.
//! drop(item_clone);
//! td.assert_no_drop(id);
//!
//! // Decrease the reference counter and then drop.
//! drop(item);
//! td.assert_drop(id);
//! ```
//!
//! Test if the [`std::vec::Vec`](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html) drop
//! implementation works.
//!
//! ```
//! use testdrop::TestDrop;
//!
//! let td = TestDrop::new();
//! let v: Vec<_> = (0..10).map(|_| td.new_item().1).collect();
//!
//! drop(v);
//!
//! // Vec::drop should drop all items.
//! assert_eq!(10, td.num_tracked_items());
//! assert_eq!(10, td.num_dropped_items());
//! ```

use std::cell::{Cell, RefCell};
use std::fmt;

/// A struct to help test drop related issues.
///
/// See the [module](index.html) documentation for examples of usage.
#[derive(Default, Debug)]
pub struct TestDrop {
    drops: Cell<usize>,
    is_dropped: RefCell<Vec<bool>>,
}

impl TestDrop {
    /// Creates a new `TestDrop`.
    pub fn new() -> TestDrop {
        TestDrop::default()
    }

    /// Creates a new [`Item`](struct.Item.html) and returns the `id` of the item and the item.
    /// The `id` of the item can be used with [`assert_drop`](struct.TestDrop#tymethod.assert_drop)
    /// and [`assert_no_drop`](struct.TestDrop#tymethod.assert_no_drop).
    pub fn new_item(&self) -> (usize, Item) {
        let id = self.num_tracked_items();
        self.is_dropped.borrow_mut().push(false);
        (id, Item::new(id, self))
    }

    /// Returns the number of tracked items.
    pub fn num_tracked_items(&self) -> usize {
        self.is_dropped.borrow().len()
    }

    /// Returns the number of dropped items so far.
    pub fn num_dropped_items(&self) -> usize {
        self.drops.get()
    }

    /// Asserts that an item was dropped.
    ///
    /// # Panics
    ///
    /// If the item was not dropped.
    pub fn assert_drop(&self, id: usize) {
        assert!(self.is_dropped(id), "{} should be dropped, but was not", id);
    }

    /// Asserts that an item was not dropped.
    ///
    /// # Panics
    ///
    /// If the item was dropped.
    pub fn assert_no_drop(&self, id: usize) {
        assert!(
            !self.is_dropped(id),
            "{} should not be dropped, but was",
            id
        );
    }

    fn is_dropped(&self, id: usize) -> bool {
        self.is_dropped.borrow()[id]
    }

    fn add_drop(&self, id: usize) {
        if self.is_dropped(id) {
            panic!("{:?} is already dropped", id)
        }
        self.is_dropped.borrow_mut()[id] = true;
        self.drops.set(self.num_dropped_items() + 1);
    }
}

/// An item tracked by `TestDrop`.
///
/// This `struct` is created by [`TestDrop::new_item`](struct.TestDrop.html). See its documentation
/// for more.
pub struct Item<'a> {
    id: usize,
    parent: &'a TestDrop,
}

impl<'a> fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Item {{ id: {} }}", self.id)
    }
}

impl<'a> PartialEq for Item<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.parent as *const _ == other.parent as *const _
    }
}

impl<'a> Drop for Item<'a> {
    fn drop(&mut self) {
        self.parent.add_drop(self.id)
    }
}

impl<'a> Item<'a> {
    fn new(id: usize, parent: &'a TestDrop) -> Self {
        Item {
            id: id,
            parent: parent,
        }
    }

    /// Returns the `id` of this item.
    pub fn id(&self) -> usize {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "0 should be dropped, but was not")]
    fn assert_drop() {
        let td = TestDrop::new();
        let (id, _item) = td.new_item();
        td.assert_drop(id);
        unreachable!();
    }

    #[test]
    #[should_panic(expected = "0 should not be dropped, but was")]
    fn assert_no_drop() {
        let td = TestDrop::new();
        let (id, item) = td.new_item();
        td.assert_no_drop(id);
        drop(item);
        td.assert_drop(id);
        td.assert_no_drop(id);
        unreachable!();
    }

    #[test]
    #[should_panic(expected = "0 is already dropped")]
    fn drop_more_than_once() {
        let td = TestDrop::new();
        let (_, a) = td.new_item();
        unsafe { ::std::ptr::read(&a as *const _) };
    }

    #[test]
    fn count() {
        let td = TestDrop::new();
        assert_eq!(0, td.num_tracked_items());
        assert_eq!(0, td.num_dropped_items());

        let (_, a) = td.new_item();
        let (_, b) = td.new_item();
        assert_eq!(2, td.num_tracked_items());
        assert_eq!(0, td.num_dropped_items());

        drop(a);
        assert_eq!(2, td.num_tracked_items());
        assert_eq!(1, td.num_dropped_items());

        drop(b);
        assert_eq!(2, td.num_tracked_items());
        assert_eq!(2, td.num_dropped_items());
    }

    #[test]
    fn item_eq() {
        let td1 = TestDrop::new();
        let (_, i1) = td1.new_item();

        let td2 = TestDrop::new();
        let (_, i2) = td2.new_item();
        let (_, i3) = td2.new_item();

        assert_eq!(i1, i1);
        assert_eq!(i2, i2);
        assert_ne!(i1, i2);
        assert_ne!(i2, i1);
        assert_ne!(i2, i3);
    }

    #[test]
    fn item_debug() {
        let td = TestDrop::new();
        let (a, item) = td.new_item();
        assert!(format!("{:?}", item).contains(&format!("id: {}", a)));
    }
}
