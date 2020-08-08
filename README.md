[![Crates.io](https://img.shields.io/crates/v/watermark.svg)](https://crates.io/crates/watermark)

# Watermark

A simple watermarking set.

A watermarking set holds any integer values, and supports two operations:

 - insert(element: T)
   - Inserts an item into the set
 - contains(element: T)
   - Checks whether an item has previously been added to the set

A watermark set works best when the "inserts" *all* happen, and happen "mostly"
in order. For example, when keeping track of which message IDs have been seen.

## Example

To make a simple idempotent data processor:

```rust
let mut ws = watermark::WatermarkSet::default();
for message in message_bus {
  if !ws.contains(message.id) {
    ws.insert(message.id);
    // Do some work with message.data
  }
}
```

## Operation

Internally, a watermarking set contains a "watermark" and a bitvector of
"recently added" items.  The watermark guarantees that all items below
that number have been seen, and the recently added items handles everything
else.  This means that if all elements eventually get added, memory usage
is kept very low and membership tests are very very cheap.

## Performance

This crate comes with some simple benchmarks that you can run for yourself.
Here are the results, compared to a hash set, that I got on my machine:

```
Specs:
 - AMD Ryzen 7 3700x 8-Core Processor
 - 2x16gb DDR4 3200
```

```
WatermarkSet Insert/In Order
                        time:   [7.6194 us 7.8101 us 7.9793 us]
                        thrpt:  [125.32 Melem/s 128.04 Melem/s 131.24 Melem/s]

HashSet Insert/In Order time:   [40.319 us 40.340 us 40.372 us]
                        thrpt:  [24.770 Melem/s 24.789 Melem/s 24.802 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe

WatermarkSet Insert/Out of Order
                        time:   [9.1897 us 9.4983 us 9.7602 us]
                        thrpt:  [102.46 Melem/s 105.28 Melem/s 108.82 Melem/s]

HashSet Insert/Out of Order
                        time:   [26.327 us 26.342 us 26.355 us]
                        thrpt:  [37.943 Melem/s 37.962 Melem/s 37.983 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

WatermarkSet Contains/Aligned
                        time:   [242.04 ns 242.25 ns 242.47 ns]
                        thrpt:  [2.6396 Gelem/s 2.6419 Gelem/s 2.6442 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

HashSet Contains/Aligned
                        time:   [7.6920 us 7.7382 us 7.8144 us]
                        thrpt:  [81.900 Melem/s 82.706 Melem/s 83.204 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

WatermarkSet Contains/Unaligned
                        time:   [290.21 ns 290.39 ns 290.56 ns]
                        thrpt:  [2.2026 Gelem/s 2.2039 Gelem/s 2.2053 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

HashSet Contains/Unaligned
                        time:   [7.9000 us 7.9021 us 7.9043 us]
                        thrpt:  [80.969 Melem/s 80.991 Melem/s 81.012 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
```
