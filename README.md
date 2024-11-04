# Safer Solana Development

This repository will warehouse code and notes that may help the common Solana developer.

**Use any packages or code examples hosted in this repository at your own risk. Currently nothing in
this repository has been formally audited.**

## Packages

- [sealevel-tools] (0.3.1): Tools for Solana program development to promote
  lightweight programs. PRs welcome.

## TODO
- Write more [examples] (like SPL mint and token management) using [sealevel-tools].
- Write [comparisons] of programs using existing Solana program frameworks as more CPI calls get
  introduced to [sealevel-tools]. Document program binary sizes and compute unit usage.
  - NOTE: These comparisons will be updated when [anchor-lang] supports Solana 2.0.
- Write about various gotchas from experience writing various programs (e.g. spooky stack access
  errors using [anchor-lang] with optimized compiles)

[anchor-lang]: https://docs.rs/anchor-lang/latest/anchor_lang/
[comparisons]: comparisons
[examples]: examples
[sealevel-tools]: sealevel-tools
