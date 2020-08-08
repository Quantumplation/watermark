//! A simple watermarking set.
//!
//! A watermarking set holds any integer values, and supports two operations:
//! 
//! - insert(element: T)
//!   - Inserts an item into the set
//! - contains(element: T)
//!   - Checks whether an item has previously been added to the set
//!
//! A watermark set works best when the "inserts" *all* happen, and happen "mostly"
//! in order. For example, when keeping track of which message IDs have been seen.
//!
//! # Example
//!
//! To make a simple idempotent data processor:
//!
//! ```rust
//!
//! struct message {
//!   id: u32,
//!   data: u64,
//! }
//!
//! let message_bus = vec![
//!   message { id: 1, data: 2 },
//!   message { id: 2, data: 3 },
//!   message { id: 1, data: 2 }
//! ];
//!
//! let mut ws = watermark::WatermarkSet::default();
//! for message in message_bus {
//!   if !ws.contains(message.id) {
//!     ws.insert(message.id);
//!     // Do some work with message.data
//!   }
//! }
//! ```
//!                         
//! # Operation
//! 
//! Internally, a watermarking set contains a "watermark" and a bitvector of
//! "recently added" items.  The watermark guarantees that all items below
//! that number have been seen, and the recently added items handles everything
//! else.  This means that if all elements eventually get added, memory usage
//! is kept very low and membership tests are very very cheap.

extern crate num;

use std::collections::VecDeque;
use num::{Integer, CheckedAdd, CheckedSub, FromPrimitive, ToPrimitive};

/// A watermarking set
/// 
/// Allows insert and contains operations.
#[derive(Default)]
pub struct WatermarkSet<T> {
    pub watermark: T,
    pub recently_added: VecDeque<u64>,
}

impl <T: Integer + CheckedSub + ToPrimitive> WatermarkSet<T> {
    fn bucket_and_offset(&self, elem: T) -> (usize, usize) {
        let diff = elem.checked_sub(&self.watermark).unwrap();
        let diff: usize = diff.to_usize().unwrap();
        // We use u64s as bitmasks for elements "just above" the water
        // so figure out which u64 the element belongs in
        let bucket = diff / 64;
        // And within that u64, which bit corresponds to the element
        let offset = diff % 64;
        return (bucket, offset);
    }
}

impl <T> WatermarkSet<T> {
    /// Create a new benchmarking set containing all elements less than the first
    /// parameter.
    /// ```
    /// let wm = watermark::WatermarkSet::new(1385);
    /// assert!(wm.contains(1384));
    /// assert_eq!(wm.contains(1385), false);
    /// assert_eq!(wm.contains(1386), false);
    /// ```
    pub fn new(watermark: T) -> WatermarkSet<T> {
        WatermarkSet {
            watermark: watermark,
            recently_added: VecDeque::default(),
        }
    }
}

impl <T: Integer + CheckedSub + CheckedAdd + FromPrimitive + ToPrimitive> WatermarkSet<T> {
    /// Insert an element to the collection
    /// # Example
    /// ```
    /// let mut wm = watermark::WatermarkSet::default();
    /// wm.insert(123);
    /// ```
    /// # Panics
    ///
    /// If the collection gets completely full, watermark may overflow the
    /// bounds of T, resulting in an unwrap panic on a checked_add.
    ///
    pub fn insert(&mut self, elem: T) {
        // It's already below the watermark, so do nothing
        if self.watermark > elem {
            return;
        }

        // Identify which bit we need to flip
        let (bucket, offset) = self.bucket_and_offset(elem);

        // make sure we have enough capacity for the bit we're about to set
        if self.recently_added.len() <= bucket {
            self.recently_added.resize(bucket + 1, 0);
        }

        // Flip the offset'th bit in the bucket to indicate this element
        // has been added
        self.recently_added[bucket] |= 1 << offset;

        // Raise the water as far as we can
        // If all the bits are set in the first bucket,
        while !self.recently_added.is_empty() && self.recently_added[0] == !0u64 {
            // We can pop it off (cheap, because VecDeque is a ring buffer)
            self.recently_added.pop_front();
            // And raise the watermark by 64
            let stride = T::from_u8(64).unwrap();
            self.watermark = self.watermark.checked_add(&stride).unwrap();
        }
    }
}

impl<T: Integer + ToPrimitive> WatermarkSet<T> {
    /// Check how many items have been added to the collection
    /// # Example
    /// ```
    /// let mut wm = watermark::WatermarkSet::default();
    /// for i in 0..=63 {
    ///     wm.insert(i);
    /// }
    /// for i in (64..80).step_by(3) {
    ///     wm.insert(i);
    /// }
    /// assert_eq!(wm.size(), 64 + 6)
    /// ```
    pub fn size(&self) -> usize {
        // Count anything that's submerged in the watermark,
        let mut size: usize = self.watermark.to_usize().unwrap();
        // Plus any bits set above the watermark
        for bucket in &self.recently_added {
            size += bucket.count_ones() as usize;
        }
        return size;
    }
}

impl<T: Integer + CheckedSub + ToPrimitive> WatermarkSet<T> {
    /// Check if an element has been added to the collection
    /// # Example
    /// ```
    /// let mut wm = watermark::WatermarkSet::default();
    /// wm.insert(1);
    /// assert!(wm.contains(1));
    /// assert_eq!(wm.contains(2), false);
    /// ```
    pub fn contains(&self, elem: T) -> bool {
        // If asked about something below the waterline, return true
        if self.watermark > elem {
            return true;
        }
        // Find out which bit to check
        let (bucket, offset) = self.bucket_and_offset(elem);
        // If we don't have a bucket for it yet, return false
        if self.recently_added.len() <= bucket {
            return false;
        }
        // Otherwise, check if the bit is set
        return self.recently_added[bucket] & (1 << offset) > 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use rand::{thread_rng, Rng};

    #[test]
    fn can_create_as_default() {
        let collection = WatermarkSet::default();
        assert_eq!(collection.contains(1), false);
        assert_eq!(collection.contains(0), false);
    }
    
    #[test]
    fn basic_operation() {
        let mut collection = WatermarkSet::default();
        collection.insert(1);
        assert!(collection.contains(1));
        assert_eq!(collection.contains(0), false);
    }

    #[test]
    fn can_check_size() {
        let mut collection = WatermarkSet::default();
        let mut rng = thread_rng();
        let mut expected_count = 0;
        let mid = rng.gen_range(10,100);
        for i in 0..mid {
            collection.insert(i);
            expected_count += 1;
        }
        let upper = rng.gen_range(mid,500);
        let step = rng.gen_range(3,20);
        for i in (mid..upper).step_by(step) {
            collection.insert(i);
            expected_count += 1;
        }
        assert_eq!(collection.size(), expected_count);
    }

    use std::panic;
    #[test]
    fn can_insert_many_with_good_watermarking() {
        let mut collection = WatermarkSet::default();
        // Insert the numbers 0 through 16383 (exclusive)
        for i in 0..(1<<14) - 1 {
            collection.insert(i);
        }
        // We should have been able to raise the watermark to 16320
        assert!(collection.watermark == (1<<14) - 64);
        // and have one entry tracking recent additions
        assert!(collection.recently_added.len() == 1);
        // And have all but the 64th bit flipped
        assert!(collection.recently_added[0] == !(1 << 63));

        // Add 16383 (the 16384th entry, starting at 0)
        collection.insert((1<<14) - 1);

        // This should have filled up the last bucket, causing
        // us to raise the watermark
        assert!(collection.watermark == (1<<14));
        assert!(collection.recently_added.len() == 0);
    }

    #[test]
    fn can_insert_slightly_out_of_order() {
        // Generate a list of IDs we're going to insert
        // that are "mostly" in order
        let mut items: Vec<u32> = (0..1<<12).collect();
        let mut rng = thread_rng();
        for i in 0..items.len() {
            // If we've swapped this item forward, leave it here
            if items[i] != i.try_into().unwrap() {
                continue;
            }
            // Swapping items with elements within 100 units of them
            // means that the whole list will be "mostly" in order
            // and items will be within 100 units of where they "should" be
            let mut offset: i32 = rng.gen_range(-100, 100);
            let idx: i32 = i.try_into().unwrap();
            let count: i32 = items.len().try_into().unwrap();
            if idx + offset < 0 {
                offset = 0;
            } else if idx + offset > count - 1 {
                offset = (count - 1) - idx;
            }
            let j: usize = (idx + offset).try_into().unwrap();
            items.swap(i, j);
        }

        let mut coll = WatermarkSet::default();
        // Now, insert each item
        for item in items {
            coll.insert(item);
        }
        // And check that we watermarked correctly to this point
        assert!(coll.watermark == (1<<12));
        assert!(coll.recently_added.len() == 0);
    }

    use num::{Num, BigUint};
    #[test]
    fn should_support_other_integer_types() {
        {
            let mut coll: WatermarkSet<u8> = WatermarkSet::default();
            // Make sure not to try 255, as that causes us to raise the
            // watermark out of bounds
            for i in 0u8..255u8 {
                coll.insert(i);
                assert!(coll.contains(i));
            }
        }
        {
            let start = BigUint::from_str_radix(
                "10000000000000000000000000000",
                10
            ).unwrap();
            let mut coll: WatermarkSet<BigUint> = WatermarkSet {
                watermark: start.clone(),
                recently_added: VecDeque::default()
            };
            for i in 0u8..255u8 {
                let elem = start.clone() + BigUint::from(i); 
                coll.insert(elem.clone());
                assert!(coll.contains(elem.clone()));
            }
        }
    }
}
