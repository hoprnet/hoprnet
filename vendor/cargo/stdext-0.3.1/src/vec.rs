//! Extension traits for `std::Vec`.

/// Extension trait with useful methods for [`std::vec::Vec`].
///
/// [`std::vec::Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
pub trait VecExt<T> {
    /// Resizes the `Vec` in-place if the provided `new_len` is **greater** than
    /// the current `Vec` length.
    ///
    /// In simple words, this method only make vector bigger, but not smaller.
    /// Calling this method with a length smaller or equal to the current length will
    /// do nothing.
    ///
    /// This method uses a closure to create new values on every push. If
    /// you'd rather [`Clone`] a given value, use [`resize_up`]. If you want
    /// to use the [`Default`] trait to generate values, you can pass
    /// [`Default::default()`] as the second argument.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let mut vec = vec![1, 2, 3];
    /// vec.resize_up_with(5, Default::default);
    /// assert_eq!(vec, [1, 2, 3, 0, 0]);
    ///
    /// let mut vec = vec![];
    /// let mut p = 1;
    /// vec.resize_up_with(4, || { p *= 2; p });
    /// assert_eq!(vec, [2, 4, 8, 16]);
    ///
    /// let mut vec = vec![1, 2, 3];
    /// vec.resize_up_with(1, Default::default);
    /// assert_eq!(vec, [1, 2, 3]); // Resizing to smaller size does nothing.
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    /// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
    /// [`Default::default()`]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
    /// [`resize_up`]: ./trait.VecExtClone.html#tymethod.resize_up
    fn resize_up_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T;

    /// Removes the first instance of `item` from the vector if the item exists.
    ///
    /// Based on the unstable vec_remove_item feature, which has been deprecated and will be removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    /// let mut vec = vec![1, 2, 3, 1];
    ///
    /// vec.remove_item(&1);
    ///
    /// assert_eq!(vec, vec![2, 3, 1]);
    /// ```
    fn remove_item<V>(&mut self, item: &V) -> Option<T>
    where
        T: PartialEq<V>;
}

/// Extension trait with useful methods for [`std::vec::Vec`].
///
/// This trait contains functions that require `T` to implement `Clone` trait.
///
/// [`std::vec::Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
pub trait VecExtClone<T: Clone> {
    /// Resizes the `Vec` in-place if the provided `new_len` is **greater** than
    /// the current `Vec` length.
    ///
    /// In simple words, this method only make vector bigger, but not smaller.
    /// Calling this method with a length smaller or equal to the current length will
    /// do nothing.
    ///
    /// This method requires `T` to implement [`Clone`],
    /// in order to be able to clone the passed value.
    /// If you need more flexibility (or want to rely on [`Default`] instead of
    /// [`Clone`]), use [`resize_up_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let mut vec = vec!["hello"];
    /// vec.resize_up(3, "world");
    /// assert_eq!(vec, ["hello", "world", "world"]);
    ///
    /// let mut vec = vec![1, 2, 3, 4];
    /// vec.resize_up(2, 0);
    /// assert_eq!(vec, [1, 2, 3, 4]); // Resizing to smaller size does nothing.
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    /// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
    /// [`resize_up_with`]: ./trait.VecExt.html#tymethod.resize_up_with
    fn resize_up(&mut self, new_len: usize, value: T);
}

impl<T> VecExt<T> for Vec<T> {
    fn resize_up_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        if self.len() < new_len {
            self.resize_with(new_len, f);
        }
    }

    fn remove_item<V>(&mut self, item: &V) -> Option<T>
    where
        T: PartialEq<V>,
    {
        let pos = self.iter().position(|x| *x == *item)?;
        Some(self.remove(pos))
    }
}

impl<T: Clone> VecExtClone<T> for Vec<T> {
    fn resize_up(&mut self, new_len: usize, value: T) {
        if self.len() < new_len {
            self.resize(new_len, value);
        }
    }
}
