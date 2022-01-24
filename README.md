# Dynamically reloadable modules in serenity

**Warning: don't use this for production-use bots (probably). It's fragile, easy to mess up
and crash, and unidiomatic**

This example shows how to implement dynamically reloadable modules for serenity Discord bots using
shared libraries and [libloading](https://docs.rs/libloading). The idea is simple: compile the
module into a shared library using `dylib` crate type and load it from the main bot process via
`libloading::Library::new()` and `library.get()`.

## How to run
Compile the module with `cargo build -p commands` and run the bot with `cargo run -p bot`. In
Discord, type `load` to load the module and then `ping` to run the function defined within
`commands/src/lib.rs`.

Now try changing the string in `commands/src/lib.rs` from `"Pong!"` to something else. Then,
unload the module by typing `unload` in Discord, recompile the module with
`cargo build -p commands`, and reload it by typing `load` in Discord. Now, when you type `ping`, the
new string should be printed!

## General instructions

General instructions for setting up dynamically loadable modules for serenity:

1. Make a crate for the reloadable module and set the crate-type to `dylib`.
    
    Note: `dylib`, not `cdylib`, because `cdylib` would require all exposed functions and types
    to be C-compatible, which is infeasable for such a complex Rust project
2. Add some code to the library

    Make sure that none of the functions you'll access later return `impl Trait` or are `async fn`:
    these make the actual return type hidden and unnameable. The main bot
    process needs to know the exact return type in order to load the function!

    Instead, return boxed trait objects, for example `BoxFuture` (see commands/src/lib.rs)
3. Add `#[no_mangle]` to all items from the library the bot needs access to

    Without this attribute, function names will be
    [mangled](https://en.wikipedia.org/wiki/Name_mangling): `ping_command` turns into
    `_ZN8commands12ping_command17h25185198a6b8b349E`. Trust me, you don't want to have to remember
    mangled names ;)
4. Make another crate for the main bot process and add [`libloading`](https://docs.rs/libloading)
    as a dependency.

    `libloading` is a cross-platform interface for dynamically loading shared libraries
5. In the main bot crate, use `libloading::Library::new()` and `library.get()` to load a library
    and access an item within

    Pass the path to the shared library to `Library::new()`. It ends with `.so` on Linux and `.dll`
    on Windows. You can find it in the `target` folder of the `dylib` crate.

    Example:
    ```rust
    unsafe {
        let lib = libloading::Library::new("target/debug/libcommands.so")?;
        // Make sure to use the correct function signature here, or you'll get UB and crashes!
        let func = lib.get::<fn(Context, Message) -> BoxFuture<Result<()>>>(b"ping")?;

        // Calling the function we just loaded!
        func(ctx, msg).await?;
    }
    ```

You will need to cache the `Library` struct because loading a shared library is very slow. In this
repo for example, the library is loaded manually using `load` and `unload` commands
(see bot/src/main.rs).

Also, make sure the main bot process and the shared library are using identical compiler versions and
identical dependency versions. To ensure the latter, you can put both crates into a single workspace
(as done in this repo).

## Why unload and load separately instead of a single reload command?

The shared library file must not be modified before the library is unloaded, because shared libraries
have destructor functions. If you change the file, the destructor functions point into garbage.

Therefore the correct procedure is: unload the module, _then_ replace the shared library, _then_
load the module again.

## What not to do

This technique is fragile and can easily cause UB and crash the bot, for example by:
- removing the shared library file while the bot is running
- not unloading the module before replacing the file
- using different compiler versions for shared library and the main bot
- using different dependency versions for shared library and the main bot
