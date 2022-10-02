# AnyMap

A specialized `HashMap` wrapper that allows to store any kind of value instead of a fixed value type.

```rust
use anymap::AnyMap;

let mut map = AnyMap::<&'static str>::new();

map.insert::<u32>("u32", 1);
map.insert::<bool>("bool", true);
map.insert::<CustomType>("custom", CustomType::default());
```
