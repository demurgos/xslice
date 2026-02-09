use crate::extent::Extent;
use crate::xslice::XsliceRef;

/// Shared extent binary string view reference
///
/// This is a specialized form of `Xslice` for `&[u8]`, with a few more
/// extra helper methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XbstrRef<'root> {
  xslice: XsliceRef<'root, u8>,
}

impl<'root> XbstrRef<'root> {
  /// Create a new root extent binary string view from the provided byte slice
  pub const fn new(root: &'root [u8]) -> Self {
    Self {
      xslice: XsliceRef::new(root),
    }
  }

  /// Get the length of this extent slice
  pub const fn len(self) -> usize {
    self.xslice.len()
  }

  /// Get a standard slice for the content of this extent slice
  pub const fn get(self) -> &'root [u8] {
    self.xslice.get()
  }

  /// If non-empty, get the first byte
  pub const fn first(self) -> Option<u8> {
    match self.xslice.first() {
      None => None,
      Some(first) => Some(*first),
    }
  }

  /// Split the extent slice at `mid`
  pub const fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
    match self.xslice.split_at_checked(mid) {
      None => None,
      Some((left, right)) => Some((Self { xslice: left }, Self { xslice: right })),
    }
  }

  /// Extract a sub-slice for the provided extent
  pub const fn sub_slice_checked(self, extent: Extent) -> Option<Self> {
    match self.xslice.sub_slice_checked(extent) {
      None => None,
      Some(xslice) => Some(Self { xslice }),
    }
  }

  /// Get the extent of this slice relative to root slice.
  pub const fn extent(self) -> Extent {
    self.xslice.extent()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn get() {
    let root = b"abcde";
    let root: XbstrRef<'_> = XbstrRef::new(root);
    assert_eq!(root.get(), b"abcde");
    let (left, right) = root.split_at_checked(3).expect("split_at_checked succeeds");
    assert_eq!(left.get(), b"abc");
    assert_eq!(right.get(), b"de");
  }
}
