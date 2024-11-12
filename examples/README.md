# Examples of `sealevel-tools` Usage

- [account-management]
- [token-management]

## Build Requirements for Reproducibility
- Solana CLI v2.1.2
  - The easiest way to change your CLI is using [agave-install]. See `agave-install -h` for more
    information on initializing a specific version.

## Tests

Run these example tests with the following command:
```sh
cargo test-sbf
```

Future versions of Solana's rust dependencies and toolchain may introduce differences in CU usage,
which is why it is important to use this workspace's [Cargo.lock] to reproduce these tests. For your
program, I encourage you to use the latest Solana version.

[Cargo.lock]: ../Cargo.lock
[agave-install]: https://crates.io/crates/agave-install
[account-management]: account-management
[token-management]: token-management