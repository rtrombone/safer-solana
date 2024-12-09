[<img alt="license" src="https://img.shields.io/github/license/rtrombone/safer-solana?logo=github" height="20">](https://github.com/rtrombone/safer-solana/blob/main/LICENSE)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sealevel-tools?logo=rust" height="20">](https://crates.io/crates/sealevel-tools)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/sealevel-tools?logo=rust" height="20">](https://docs.rs/sealevel-tools)

# Sealevel Tools

Tools for safer and CU-optimized Solana development.

This crate is not an attempt to create a new framework for writing Solana
programs. Instead, it is a set of tools that should help a developer write a
Solana program without prescribing any specific way of doing so. By using these
tools, a developer can write a lightweight program with functionality found in
other frameworks.

See crate [documentation] for more information.

## Dependencies

Minimum-supported Rust version: **1.79**, which matches MSRV of Solana-related
crates with version ^2.1.4. Keep in mind that with future Solana versions, MSRV
may uptick.

Currently, this package leverages [sealevel-nostd-entrypoint], which is a fork
of [solana-nostd-entrypoint] (an optimized no-std program entrypoint library)
updated to use Solana ^2.0. Its contents are re-exported for convenience into
the [entrypoint] submodule.

## Feature Flags

When you add this package, the following features are enabled by default:
```toml
default = [
    "alloc",
    "borsh",
    "token"
]
```

To disable these defaults (e.g. using a heapless environment via
[noalloc_allocator]), use `default-features = false` in your Cargo.toml and add
the features you need for your program:
```toml
sealevel-tools = { version = "0.7", default-features = false, features = ["noalloc-default"] }
```

### `features = ["alloc"]`

Using the Rust's core allocation library, enable this feature. If this feature
is disabled, be aware that error resolution decreases. For example, with alloc
on, you may encounter a program log resembling:
```console
Program log: Custom error: AccountInfo
Program log: Account key mismatch at index 1...
Program log:   Found: 7UbHLbKLfvh3maXuCZMWqKjzMCeLczaJwWTSLVpYV38z
Program log:   Expected: CjasN94JjDrDeZkxenJGNrN1sqBeHauNkJDp48VQdrtm
```

**NOTE: With the above example, any referenced index is based on zero-indexed
account enumeration.**

And without it, the same error would produce:
```console
Program log: Custom error: AccountInfo
Program log: Account does not match expected key
```

Having no allocator is extreme. You can still write a program that does not
allocate to the heap and keep this feature enabled so that the program logs have
more colorful error messages. There would only be a cost to process these errors
if preflight were skipped when sending the transaction and you wanted these
failed transactions to persist on-chain.

See [alloc] documentation for more information.

### `features = ["borsh"]`

Account handling relating to [borsh] serialization. It is up to you to determine
whether there is too much overhead to use this serialization library and whether
it is worth the time to write your own serde.

It should be no cost to have this feature enabled. And as an integrator with
another program, you will have the option to deserialize accounts of programs
written with frameworks like [anchor-lang].

### `features = ["token"]`

Account and CPI handling relating to the SPL Token programs (legacy and
Extensions).

There are convenient methods that create mints and token accounts, which
basically pair a System program's create account with an initialize mint or
initialize token account instruction to allow these operations to happen
atomically in your program's instruction (as opposed to having to create an
account in an instruction prior to invoking your program's).

## Philosophy

The tools found in this crate are meant to allow a developer to keep things as
simple as possible while providing some guardrails. These guardrails are not
meant to enforce any specific way of iterating through entrypoint accounts,
account serialization/deserialization (serde), how instruction discriminators
should be set, etc.

The developer should write a program without any artificial constraints. For
example, a developer may want instruction selectors to be the first 4-bytes of a
Keccak256 hash similar to how Solidity for EVM works. Or to be consistent with
how [anchor-lang] and [spl-discriminator] define discriminators as the first
8-bytes of Sha256 (sha2).

What this crate does not attempt to do is generate an IDL. While convenient when
using frameworks like [anchor-lang] and [shank] (where a front-end
language-agnostic developer can take an IDL and build an SDK using it), these
tools are meant to focus on safer program development. Writing instruction
builders is trivial, and any time spent trying to resolve headaches having an
IDL build using any of these frameworks can be saved by writing custom
instruction builders in whichever language you want to support.

Solana program frameworks attempt to remove boilerplate from writing instruction
processors. But with that comes the price of having to learn how these specific
macros work. And these macros can add a lot of bloat to your program, where your
program size can easily be double the size of a program with the same logic but
without a specific framework.

[alloc]:  https://doc.rust-lang.org/alloc/
[anchor-lang]: https://crates.io/crates/anchor-lang/
[borsh]: https://crates.io/crates/borsh/
[documentation]: https://docs.rs/sealevel-tools/
[entrypoint]: https://docs.rs/sealevel-tools/latest/sealevel_tools/entrypoint/index.html
[noalloc_allocator]: https://docs.rs/sealevel-nostd-entrypoint/0.1.0/sealevel_nostd_entrypoint/macro.noalloc_allocator.html
[sealevel-nostd-entrypoint]: https://crates.io/crates/sealevel-nostd-entrypoint/
[solana-nostd-entrypoint]: https://crates.io/crates/solana-nostd-entrypoint/
[spl-discriminator]: https://crates.io/crates/spl-discriminator/
[shank]: https://crates.io/crates/shank/