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
        // - we ensure that there's no other exclusive reference at the same time using the type system
        unsafe { core::slice::from_raw_parts(start, len) }
      }
    }
  }

  /// If non-empty, get a reference to the first item
  pub const fn first(self) -> Option<&'root T> {
    self.get().first()
  }

  /// If non-empty, get a reference to the last item
  pub const fn last(self) -> Option<&'root T> {
    self.get().last()
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

  /// If non-empty, get the first item and rest of the slice
  pub const fn split_first_checked(self) -> Option<(&'root T, Self)> {
    match self.extent.pop_start() {
      None => None,
      Some(rest_extent) => {
        let first = self
          .const_copy()
          .first()
          .expect("`first` exists if popping the extent succeeds");
        Some((
          first,
          Self {
            root: self.root,
            extent: rest_extent,
            phantom: self.phantom,
          },
        ))
      }
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

  /// Extract a sub-slice for the first `count` items
  pub const fn sub_slice_to_checked(self, count: usize) -> Option<Self> {
    match self.split_at_checked(count) {
      None => None,
      Some((left, _)) => Some(left),
    }
  }

  /// Extract a sub-slice for the items after index `count`
  pub const fn sub_slice_from_checked(self, count: usize) -> Option<Self> {
    match self.split_at_checked(count) {
      None => None,
      Some((_, right)) => Some(right),
    }
  }

  /// Get the extent of this slice relative to root slice.
  pub const fn extent(self) -> Extent {
    self.extent
  }

  pub const fn const_copy(&self) -> Self {
    Self { ..*self }
  }

  /// Reborrow with a shorter lifetime
  pub const fn reborrow<'short>(&'short self) -> XsliceRef<'short, T>
  where
    'root: 'short,
  {
    XsliceRef {
      root: self.root,
      extent: self.extent,
      phantom: PhantomData,
    }
  }
}

/// Exclusive (mutable) extent slice reference
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XsliceMut<'root, T> {
  root: *mut T,
  extent: Extent,
  phantom: PhantomData<&'root mut [T]>,
}

impl<'root, T> XsliceMut<'root, T> {
  /// Create a new exclusive root extent slice from the provided standard slice
  pub const fn new(root: &'root mut [T]) -> Self {
    let extent = Extent::new_to(root.len());
    let root: *mut T = root.as_mut_ptr();
    Self {
      root,
      extent,
      phantom: PhantomData,
    }
  }

  /// Get a standard exclusive slice ref for the content of this extent slice
  pub const fn get_mut(self) -> &'root mut [T] {
    match self.extent.start() {
      None => unsafe { core::slice::from_raw_parts_mut(self.root, 0) },
      Some(start) => {
        let start: *mut T = self.root.wrapping_add(start);
        let len = self.extent.len();
        // SAFETY:
        // - `self.root` was created from a `&'root mut [T]` slice, let's call it `root_slice`
        // - At all times, `self.extent.start().unwrap_or(0) <= root_slice.len()`
        // - At all times, `self.extent.start().unwrap_or(0) + self.extent.len() <= root_slice.len()`
        // - `self.root` and `start` have the same provenance
        // - we ensure that there's no other exclusive reference at the same time using the type system
        unsafe { core::slice::from_raw_parts_mut(start, len) }
      }
    }
  }

  /// If non-empty, get an exclusive reference to the first item
  pub const fn first_mut(self) -> Option<&'root mut T> {
    self.get_mut().first_mut()
  }

  pub const fn as_ref(self) -> XsliceRef<'root, T> {
    XsliceRef {
      root: self.root.cast_const(),
      extent: self.extent,
      phantom: PhantomData,
    }
  }

  /// Reborrow with a shorter lifetime
  pub const fn reborrow_mut<'short>(&'short mut self) -> XsliceMut<'short, T>
  where
    'root: 'short,
  {
    XsliceMut {
      root: self.root,
      extent: self.extent,
      phantom: PhantomData,
    }
  }

  /// Alias for `.reborrow_mut().get_mut()`
  pub const fn rb_get_mut<'short>(&'short mut self) -> &'short mut [T]
  where
    'root: 'short,
  {
    self.reborrow_mut().get_mut()
  }

  /// Alias for `.reborrow_mut().first_mut()`
  pub const fn rb_first_mut<'short>(&'short mut self) -> Option<&'short mut T>
  where
    'root: 'short,
  {
    self.reborrow_mut().first_mut()
  }

  /// Alias for `.reborrow_mut().as_ref()`
  pub const fn rb_as_ref<'short>(&'short mut self) -> XsliceRef<'short, T>
  where
    'root: 'short,
  {
    self.reborrow_mut().as_ref()
  }

  /// Alias for `.reborrow_mut().as_ref().get()`
  pub const fn rb_get<'short>(&'short mut self) -> &'short [T]
  where
    'root: 'short,
  {
    self.reborrow_mut().as_ref().get()
  }
}

#[cfg(feature = "reborrow05")]
impl<'short, 'root, T> reborrow05::Reborrow<'short> for XsliceRef<'root, T> {
  type Target = XsliceRef<'short, T>;

  fn rb(&'short self) -> Self::Target {
    self.reborrow()
  }
}

#[cfg(feature = "reborrow05")]
impl<'short, 'root, T> reborrow05::ReborrowMut<'short> for XsliceMut<'root, T> {
  type Target = XsliceMut<'short, T>;

  fn rb_mut(&'short mut self) -> Self::Target {
    self.reborrow_mut()
  }
}

#[cfg(feature = "reborrow05")]
impl<'root, T> reborrow05::IntoConst for XsliceMut<'root, T> {
  type Target = XsliceRef<'root, T>;

  fn into_const(self) -> Self::Target {
    self.as_ref()
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

  #[test]
  fn first_mut() {
    let mut roota = [10, 11, 12, 13];
    let mut root: XsliceMut<'_, i32> = XsliceMut::new(&mut roota);
    {
      assert_eq!(root.rb_get_mut(), &[10, 11, 12, 13]);
    }
    let first = root.rb_first_mut().expect("first_mut succeeds");
    *first = 20;
    assert_eq!(root.rb_get_mut(), &[20, 11, 12, 13]);
    let second = root.rb_first_mut().expect("first_mut succeeds");
    *second = 30;
    // *first = 40; // THIS SHOULD BE BLOCKED AT COMPILE TIME
    assert_eq!(root.rb_get_mut(), &[30, 11, 12, 13]);
    assert_eq!(root.rb_get(), &[30, 11, 12, 13]);
    assert_eq!(root.rb_as_ref().get(), &[30, 11, 12, 13]);
    assert_eq!(root.rb_get(), &[30, 11, 12, 13]);
    roota[0] = 40;
    assert_eq!(roota, [40, 11, 12, 13]);
  }
}
