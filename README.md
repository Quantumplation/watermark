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
