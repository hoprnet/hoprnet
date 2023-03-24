use crate::common::{wrapping_add, wrapping_sub};

pub struct Iter<'a, Item: 'a, const CAPACITY: usize> {
	items: &'a [Item],
	cursor_idx: usize,
	num_items_left: usize,
}

impl<'a, Item: 'a, const CAPACITY: usize> Iter<'a, Item, CAPACITY> {
	pub fn new(items: &'a [Item], end_idx: usize, num_items: usize) -> Self {
		Self {
			items,
			cursor_idx: wrapping_sub::<CAPACITY>(end_idx, num_items),
			num_items_left: num_items,
		}
	}
}

impl<'a, Item, const CAPACITY: usize> Iterator for Iter<'a, Item, CAPACITY> {
	type Item = &'a Item;

	fn next(&mut self) -> Option<Self::Item> {
		if self.num_items_left == 0 {
			return None;
		}

		self.num_items_left -= 1;

		let cursor_idx = self.cursor_idx;
		self.cursor_idx = wrapping_add::<CAPACITY>(self.cursor_idx, 1);

		Some(&self.items[cursor_idx])
	}
}
