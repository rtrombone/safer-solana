# Safer Solana Development

This repository will warehouse code and notes that may help the common Solana
developer.

**Use any packages or code examples hosted in this repository at your own risk.
Currently nothing in this repository has been formally audited.**

## Table of Contents
1. [Notes](#notes)
2. [Packages](#packages)
3. [TODO](#todo)

## Notes

Collected information about things I have encountered when developing Solana
programs.

- [Compute Units] -- Information about CU usage.
- [Entrypoint] -- Packages that allow a developer to work around [solana-program]
  to implement a more CU-efficient program entrypoint.

## Packages

- [sealevel-tools] -- Tools for Solana program development to promote
  lightweight programs. PRs welcome. You can find examples of its usage in
  [examples].

## TODO
- Write more [examples] (like SPL mint and token management) using
  [sealevel-tools].
- Write [comparisons] of programs using existing Solana program frameworks as
  more CPI calls get introduced to [sealevel-tools]. Document program binary
  sizes and compute unit usage.
  - NOTE: These comparisons will be updated when [anchor-lang] supports Solana
    2.0.
- Write about various gotchas from experience writing various programs (e.g.
spooky stack access errors using [anchor-lang] with optimized compiles)

[Compute Units]: COMPUTE_UNITS.md
[Entrypoint]: ENTRYPOINT.md
[anchor-lang]: https://docs.rs/anchor-lang/latest/anchor_lang/
[comparisons]: comparisons
[examples]: examples
[sealevel-tools]: sealevel-tools
