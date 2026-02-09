use crate::extent::Extent;
use crate::xbstr::XbstrRef;

/// Shared extent UTF-8 string view reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XstrRef<'root> {
  xbstr: XbstrRef<'root>,
}

impl<'root> XstrRef<'root> {
  /// Create a new root extent UTF-8 string view from the provided str view
  pub const fn new(root: &'root str) -> Self {
    Self {
      xbstr: XbstrRef::new(root.as_bytes()),
    }
  }

  /// Get the length of this extent string view, in bytes
  pub const fn len(self) -> usize {
    self.xbstr.len()
  }

  /// Get a standard str view for the content of this extent string view
  pub const fn get(self) -> &'root str {
    let bytes = self.xbstr.get();
    // SAFETY:
    // - the root slice was created from `&'root str`
    // - all splits are along character boundaries
    unsafe { core::str::from_utf8_unchecked(bytes) }
  }

  /// Check if `index` is a char boundary
  pub const fn is_char_boundary(self, index: usize) -> bool {
    if index == 0 {
      return true;
    }
    if index >= self.len() {
      index == self.len()
    } else {
      u8_is_utf8_char_boundary(self.xbstr.get()[index])
    }
  }

  /// Split the extent string view at `mid`
  ///
  /// Returns `None` if `mid` is out of bounds or not a char boundary.
  pub const fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
    if self.is_char_boundary(mid) {
      match self.xbstr.split_at_checked(mid) {
        None => None,
        Some((left, right)) => Some((Self { xbstr: left }, Self { xbstr: right })),
      }
    } else {
      None
    }
  }

  /// Get the extent of this slice relative to root str.
  pub const fn extent(self) -> Extent {
    self.xbstr.extent()
  }

  /// Get the inner extent binary string slice
  pub const fn as_xbstr(self) -> XbstrRef<'root> {
    self.xbstr
  }
}

// imported from the std lib function `u8::is_utf8_char_boundary`
#[inline]
const fn u8_is_utf8_char_boundary(byte: u8) -> bool {
  // This is bit magic equivalent to: b < 128 || b >= 192
  (byte as i8) >= -0x40
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn get() {
    let root = "abcde";
    let root: XstrRef<'_> = XstrRef::new(root);
    assert_eq!(root.get(), "abcde");
    let (left, right) = root.split_at_checked(3).expect("split_at_checked succeeds");
    assert_eq!(left.get(), "abc");
    assert_eq!(right.get(), "de");
  }
}
