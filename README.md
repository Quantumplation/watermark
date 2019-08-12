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
Here are the results, compared to a hash set, that I got on my laptop:

```
WatermarkSet Insert/In Order
                        time:   [12.528 us 13.191 us 13.863 us]
                        thrpt:  [72.135 Melem/s 75.807 Melem/s 79.819 Melem/s]

HashSet Insert/In Order time:   [52.123 us 52.573 us 53.071 us]
                        thrpt:  [18.843 Melem/s 19.021 Melem/s 19.186 Melem/s]

WatermarkSet Insert/Out of Order
                        time:   [20.463 us 21.415 us 22.209 us]
                        thrpt:  [45.027 Melem/s 46.696 Melem/s 48.870 Melem/s]

HashSet Insert/Out of Order
                        time:   [35.186 us 35.563 us 36.051 us]
                        thrpt:  [27.738 Melem/s 28.119 Melem/s 28.420 Melem/s]

WatermarkSet Contains/Aligned
                        time:   [482.96 ns 492.02 ns 502.25 ns]
                        thrpt:  [1.2743 Gelem/s 1.3008 Gelem/s 1.3252 Gelem/s]

HashSet Contains/Aligned
                        time:   [11.687 us 11.943 us 12.255 us]
                        thrpt:  [52.223 Melem/s 53.589 Melem/s 54.761 Melem/s]

WatermarkSet Contains/Unaligned
                        time:   [621.99 ns 674.32 ns 726.55 ns]
                        thrpt:  [880.87 Melem/s 949.11 Melem/s 1.0290 Gelem/s]

HashSet Contains/Unaligned
                        time:   [13.132 us 13.431 us 13.801 us]
                        thrpt:  [46.373 Melem/s 47.651 Melem/s 48.737 Melem/s]
```
