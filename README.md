# validated_newtype

Simple checked newtype generator, primarily for use with [serde](https://serde.rs).
Serde support (and dependency) may be disabled with `default_features = false`.
This is `#![no_std]` library.

Usage:
```rust
validated_newtype! {
    /// Documentation comments and attributes are applied
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
    // base type name => optional visibility + newtype name
    u32 => pub Percent
    // condition to check when creating/deserializing
    if |n: &u32| *n <= 100;
    // error message if condition is not met
    error "percent must be in range 0-100"
}

let x: Percent = serde_json::from_str("42").unwrap();
assert_eq!(*x, 42);
let y: Result<Percent, _> = serde_json::from_str("1337");
assert!(y.is_err());
```
Instances of generated newtype can be created only via [TryFrom] or [Deserialize],
so they always hold valid data.

### Dynamic error generation
```rust
validated_newtype! {
    #[derive(Debug)]
    u32 => pub Percent
    if |n: &u32| *n <= 100;
    else |n: &u32| format!("number {} is not in range 0-100", n) => String
}

// Deserialize for newtypes uses try_into internally
let x: Result<Percent, _> = 1337.try_into();
assert!(x.is_err());
assert_eq!(x.unwrap_err(), "number 1337 is not in range 0-100");
```
### Manually implement [TryFrom]
```rust
validated_newtype! {
    #[derive(Debug)]
    u32 => pub Percent
}

impl TryFrom<u32> for Percent {
    type Error = &'static str;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        if val > 100 {
            Err("percent must be in range 0-100")
        } else {
            Ok(Self(val))
        }
    }
}

let x: Percent = serde_json::from_str("42").unwrap();
assert_eq!(*x, 42);
let y: Result<Percent, _> = serde_json::from_str("1337");
assert!(y.is_err());
```

[TryFrom]: https://doc.rust-lang.org/stable/core/convert/trait.TryFrom.html
[Deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html

License: MIT
