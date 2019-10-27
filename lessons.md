# Lessons and ideas learned from this project

## Rust-specific

- Creating a structure that takes ownership of another one on creation and releases it on destruction. This can be used to manage states nicely, since the new struct can have completely different methods and the inner struct is unaccesible. This also supports nice method-chained syntax.
- Using *'cargo doc'* command to debug documentation.
- One can define additional generic trait names for trait-bounding purposes.
