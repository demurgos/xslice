use std::num::{NonZero, NonZeroUsize};

/// An extent is equivalent to a standard `Range<usize>`, but uses a different
/// internal representation to statically enforce that the bounds are in the
/// right order
///
/// It internally stores a `start` and `len` value, which are both `usize`.
/// Note that `start` is only defined if `len` is non-zero.
///
/// Extent is checked to have the same size as two `usize` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Extent {
  inner: ExtentInner,
}

impl Extent {
  /// The empty extent
  ///
  /// - `len` is zero
  /// - `start` is undefined
  pub const EMPTY: Self = Self {
    inner: ExtentInner::Empty,
  };

  /// Create a new extent for the range `0..len`
  pub const fn new_to(len: usize) -> Self {
    Self::new(0, len)
  }

  /// Create a new extent starting at `start`, with length `len`
  ///
  /// If `len` is zero, `start` is dropped.
  pub const fn new(start: usize, len: usize) -> Self {
    Self {
      inner: match NonZeroUsize::new(len) {
        Some(len) => ExtentInner::NonEmpty(Extent1 { start, len }),
        None => ExtentInner::Empty,
      },
    }
  }

  /// Get the start bound
  ///
  /// If `len` is zero, returns `None`
  pub const fn start(self) -> Option<usize> {
    match self.inner {
      ExtentInner::Empty => None,
      ExtentInner::NonEmpty(extent) => Some(extent.start),
    }
  }

  /// Get the start bound, or zero if undefined
  pub const fn start_or_zero(self) -> usize {
    match self.inner {
      ExtentInner::Empty => 0,
      ExtentInner::NonEmpty(extent) => extent.start,
    }
  }

  /// Get the length of the extent
  pub const fn len(self) -> usize {
    match self.inner {
      ExtentInner::Empty => 0,
      ExtentInner::NonEmpty(extent) => extent.len.get(),
    }
  }

  /// Split an extent at `mid`
  ///
  /// `mid` must verify `0 <= mid <= len`. If this property fails, returns
  /// `None`
  pub const fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
    match self.len().checked_sub(mid) {
      None => None,
      Some(right_len) => Some((
        Self {
          inner: match NonZeroUsize::new(mid) {
            Some(len) => ExtentInner::NonEmpty(Extent1 {
              start: self.start_or_zero(),
              len,
            }),
            None => ExtentInner::Empty,
          },
        },
        Self {
          inner: match NonZeroUsize::new(right_len) {
            Some(len) => ExtentInner::NonEmpty(Extent1 {
              start: match self.start_or_zero().checked_add(mid) {
                Some(start) => start,
                None => panic!(
                  "unreachable: `start + mid <= start + len` ans `start + len` is always a valid `usize` by construction"
                ),
              },
              len,
            }),
            None => ExtentInner::Empty,
          },
        },
      )),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExtentInner {
  Empty,
  NonEmpty(Extent1),
}

/// A non-empty range represent as a start offset and length.
///
/// This is equivalent to the standard range `start..start+len`, but it ensure
/// "by design" that the bounds are in-order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Extent1 {
  start: usize,
  len: NonZero<usize>,
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn extent_size() {
    assert_eq!(size_of::<Extent>(), 2 * size_of::<usize>());
  }
}
