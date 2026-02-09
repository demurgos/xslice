use crate::extent::Extent;
use core::marker::PhantomData;

/// Shared extent slice reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XsliceRef<'root, T> {
  root: *const T,
  extent: Extent,
  phantom: PhantomData<&'root [T]>,
}

impl<'root, T> XsliceRef<'root, T> {
  pub const EMPTY: Self = Self::new(&[]);

  /// Create a new root extent slice from the provided standard slice
  pub const fn new(root: &'root [T]) -> Self {
    let extent = Extent::new_to(root.len());
    let root: *const T = root.as_ptr();
    Self {
      root,
      extent,
      phantom: PhantomData,
    }
  }

  /// Get the length of the extent
  pub const fn len(self) -> usize {
    self.extent.len()
  }

  /// Get a standard slice for the content of this extent slice
  pub const fn get(self) -> &'root [T] {
    match self.extent.start() {
      None => &[],
      Some(start) => {
        let start: *const T = self.root.wrapping_add(start);
        let len = self.extent.len();
        // SAFETY:
        // - `self.root` was created from a `&'root [T]` slice, let's call it `root_slice`
        // - At all times, `self.extent.start().unwrap_or(0) <= root_slice.len()`
        // - At all times, `self.extent.start().unwrap_or(0) + self.extent.len() <= root_slice.len()`
        // - `self.root` and `start` have the same provenance
        unsafe { core::slice::from_raw_parts(start, len) }
      }
    }
  }

  /// If non-empty, get a referemce to the first item
  pub const fn first(self) -> Option<&'root T> {
    self.get().first()
  }

  /// Split the extent slice at `mid`
  pub const fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
    match self.extent.split_at_checked(mid) {
      None => None,
      Some((left, right)) => Some((
        Self {
          root: self.root,
          extent: left,
          phantom: self.phantom,
        },
        Self {
          root: self.root,
          extent: right,
          phantom: self.phantom,
        },
      )),
    }
  }

  /// Extract a sub-slice for the provided extent
  pub const fn sub_slice_checked(self, extent: Extent) -> Option<Self> {
    match extent.start() {
      None => Some(Self::EMPTY),
      Some(start) => match self.split_at_checked(start) {
        None => None,
        Some((_, right)) => match right.split_at_checked(extent.len()) {
          None => None,
          Some((left, _)) => Some(left),
        },
      },
    }
  }

  /// Get the extent of this slice relative to root slice.
  pub const fn extent(self) -> Extent {
    self.extent
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn get() {
    let root = &[10, 11, 12, 13];
    let root: XsliceRef<'_, i32> = XsliceRef::new(root);
    assert_eq!(root.get(), &[10, 11, 12, 13]);
    let (left, right) = root.split_at_checked(3).expect("split_at_checked succeeds");
    assert_eq!(left.get(), &[10, 11, 12]);
    assert_eq!(right.get(), &[13]);
  }
}
