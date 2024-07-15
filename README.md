# auto-from

​	auto-from is a procedural macro crate to help you automatically implement the `From` trait for enums. Its typical usage is for error enums which contain multiple kinds of error types, for example,

```rust
#[auto_from]
enum Error {
 	IOError(std::io::Error),
  FmrError(std::fmt::Error)
}
```

The code generated will be 

```rust
enum Error {
    IOError(std::io::Error),
    FmtError(std::fmt::Error),
}
impl From<std::io::Error> for Error {
    fn from(item: std::io::Error) -> Self {
        Self::IOError(item)
    }
}
impl From<std::fmt::Error> for Error {
    fn from(item: std::fmt::Error) -> Self {
        Self::FmtError(item)
    }
}
```

​	If you wish some fields to not to be implemented, you just specify them in the macro attributes, for example,

```rust
#[auto_from(disable = [IOError])]
```

​	This crate is pretty simple, as it is mainly created to show you how to write a Rust procedural macro crate. Here's a [tutorial](tutorial.md) in Chinese.