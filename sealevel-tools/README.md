[<img alt="license" src="https://img.shields.io/github/license/rtrombone/safer-solana?logo=github" height="20">](https://crates.io/sealevel-tools)[<img alt="crates.io" src="https://img.shields.io/crates/v/sealevel-tools?logo=rust" height="20">](https://crates.io/sealevel-tools)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/sealevel-tools?logo=rust" height="20">](https://docs.rs/sealevel-tools)

# Sealevel Tools

This crate is not an attempt to create a new framework for writing Solana programs. Instead, it
is a set of tools that should help a developer write a Solana program without prescribing any
specific way of doing so. By using these tools, a developer can write a lightweight program with
functionality found in other frameworks.

## Dependencies

Currently only supports Solana version ^1.18. Until [solana-nostd-entrypoint] supports a higher
version, this package will pin Solana dependencies to the above version.

## Philosophy

The tools found in this crate are meant to allow a developer to keep things as simple as possible
while providing some guardrails. These guardrails are not meant to enforce any specific way of
iterating through entrypoint accounts, account serialization/deserialization (serde), how
instruction discriminators should be set, etc.

The developer should write a program without any artificial constraints. For example, a developer
may want instruction selectors to be the first 4-bytes of a Keccak256 hash similar to how Solidity
for EVM works. Or to be consistent with how [anchor-lang] and [spl-discriminator] define
discriminators as the first 8-bytes of Sha256 (sha2).

What this crate does not attempt to do is generate an IDL. While convenient when using frameworks
like [anchor-lang] and [shank] (where a front-end language-agnostic developer can take an IDL and
build an SDK using it), these tools are meant to focus on safer program development. Writing
instruction builders is trivial, and any time spent trying to resolve headaches having an IDL build
using any of these frameworks can be saved by writing custom instruction builders in whichever
language you want to support.

Solana program frameworks attempt to remove boilerplate from writing instruction processors. But
with that comes the price of having to learn how these specific macros work. And these macros can
add a lot of bloat to your program, where your program size can easily be double the size of a
program with the same logic but without a specific framework.

[anchor-lang]: https://docs.rs/anchor-lang/latest/
[solana-nostd-entrypoint]: https://docs.rs/solana-nostd-entrypoint/
[spl-discriminator]: https://docs.rs/spl-discriminator/latest/
[shank]: https://docs.rs/shank/