# dynlib
Simple Bindings to the Microsoft DLL Loader for Rust MSVC

[Docs](https://valarauca.github.io/dynlib/dynlib/index.html)

To include in your project:

```
[dependencies]
dynlib = "0.0.1"
```

This crate WILL NOT WORK outside of Rust MSVC. Please note that.

This provides simple abstraction of the `LoadLibraryExA` interface found
on Windows platforms. A very simple example of this crates functionality


```rust

use dynlib::{VoidPtr,LoadWinDynLib,DynLibWin};

let lib: DynLibWin = LoadWinDynLib::new()
			.search_application_dir()
			.load("test_dll.dll")?;

let func: VoidPtr = lib.load_function("addition")?;
let callable: extern "Rust" fn(u64,u64)->u64 = unsafe{ mem::transmute(func)};
assert_eq( callable(5u64,5u64), 10u64);
```

Unsafe code is required to cast the function pointer. The `DynLibWin` type
will not unload it's module when it is dropped. It must be manually freed.
When you free the `DynLibWin` all functions that were pulled from it will
be invalidated, and freed from memory. So calling them will results in 
a memory segmentation fault, and likely your application crashing.

I just feel it is a safer alternative to have DLL's live for your entire
application's life. The loaded DLL's memory is shared between processes, so
there isn't a _save ram_ argument here.
