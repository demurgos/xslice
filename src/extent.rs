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

  /// Get the end bound
  ///
  /// If `len` is zero, returns `None`
  pub const fn end(self) -> Option<NonZeroUsize> {
    match self.inner {
      ExtentInner::Empty => None,
      ExtentInner::NonEmpty(extent) => extent.len.checked_add(extent.start),
    }
  }

  /// Get the length of the extent
  pub const fn len(self) -> usize {
    match self.inner {
      ExtentInner::Empty => 0,
      ExtentInner::NonEmpty(extent) => extent.len.get(),
    }
  }

  /// Remove `count` items from the start of the extent.
  ///
  /// This function checks that `count <= len`
  pub const fn strip_start_checked(self, count: usize) -> Option<Self> {
    if count == 0 {
      return Some(self);
    }
    match self.split_at_checked(count) {
      None => None,
      Some((_, right)) => Some(right),
    }
  }

  /// Remove `count` items from the end of the extent.
  ///
  /// This function checks that `count <= len`
  pub const fn strip_end_checked(self, count: usize) -> Option<Self> {
    if count == 0 {
      return Some(self);
    }
    match self.split_at_checked(count) {
      None => None,
      Some((left, _)) => Some(left),
    }
  }

  /// Reduce the extent size by removing the first item
  ///
  /// (increase `start` by one, reduce `len` by one).
  /// This is equivalent to `self.strip_start_checked(1)`.
  pub const fn pop_start(self) -> Option<Self> {
    match self.inner {
      ExtentInner::Empty => None,
      ExtentInner::NonEmpty(inner) => Some(Self {
        inner: match NonZeroUsize::new(
          inner
            .len
            .get()
            .checked_sub(1)
            .expect("decrementing non-zero always succeeds"),
        ) {
          Some(len) => ExtentInner::NonEmpty(Extent1 {
            start: inner
              .start
              .checked_add(1)
              .expect("incrementing `start` always succeeds when new `len` is non-zero"),
            len,
          }),
          None => ExtentInner::Empty,
        },
      }),
    }
  }

  /// Reduce the extent size by removing the last item
  ///
  /// (keep `start` as-is, reduce `len` by one).
  /// This is equivalent to `self.strip_end_checked(1)`.
  pub const fn pop_end(self) -> Option<Self> {
    match self.inner {
      ExtentInner::Empty => None,
      ExtentInner::NonEmpty(inner) => Some(Self {
        inner: match NonZeroUsize::new(
          inner
            .len
            .get()
            .checked_sub(1)
            .expect("decrementing non-zero always succeeds"),
        ) {
          Some(len) => ExtentInner::NonEmpty(Extent1 {
            start: inner.start,
            len,
          }),
          None => ExtentInner::Empty,
        },
      }),
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
