# Program Entrypoint

Without any framework, a developer would write a method using [solana-program]
that takes an `AccountInfo` slice for account processing. Bare-minimum example:
```rs
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey
};

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> ProgramResult {
    // TODO: Check your program ID.
    let _ = program_id;

    // TODO: Check and use your accounts.
    let _ = accounts;

    // TODO: Check and use your data.
    let _ = data;

    solana_program::log::sol_log("Hello, world!");
}
```

You can find examples of how instruction processors are written in the
[solana-program-library] repository.

## Anchor

The [anchor-lang] framework is what the average Solana program developer seeks
out (and has the most documentation written about it despite not having a
README for its crate).

An Anchor program's entrypoint is obfuscated in its program macro, which takes
all methods written to take in a `Context` struct and builds an entrypoint
method, which uses an 8-byte instruction selector for each of your methods.

If you cannot be bothered by writing your own entrypoint method and want to
focus as much on the business logic of your program, use Anchor. But if you want
a more optimized program (being conscious of its compute unit usage), read about
[optimized solutions](#optimized-solutions).

## Optimized Solutions

In alphabetical order:

- [pinocchio]\: Maintained by @febo. Has zero dependencies.
  
  A benefit of using this package is it does not depend on any version of
  Solana (so your program can adopt Solana 2.0 very easily by using it). But the
  trade-off is not being able to use [solana-program] alongside it because
  everything in this package was written without it (for example, using the
  `Pubkey` from its `AccountInfo` does not interop with the `Pubkey` found in
  [solana-program] due to being two different types).

- [solana-nostd-entrypoint]\: Maintained by @cavemanloverboy. It uses
  [solana-program] as a dependency (and re-exports it).
  
  A benefit of using this package is someone can use [solana-program] and
  leverage its structs like `NoStdAccountInfo` (see this [Helius article] that
  demonstrates its usage). But the current trade-off is it **currently only
  supports version ^1.18**.

  Also, [sealevel-tools] currently builds on top of a fork of this package
  called [sealevel-nostd-entrypoint] (which has been updated to use Solana 2.0)
  for account processing and performing CPI.

[Helius article]: https://www.helius.dev/blog/optimizing-solana-programs
[anchor-lang]: https://crates.io/crates/anchor-lang
[pinocchio]: https://crates.io/crates/pinocchio
[sealevel-nostd-entrypoint]: https://crates.io/crates/sealevel-nostd-entrypoint
[sealevel-tools]: https://crates.io/crates/sealevel-tools
[solana-nostd-entrypoint]: https://crates.io/crates/solana-nostd-entrypoint
[solana-program]: https://crates.io/crates/solana-program
[solana-program-library]: https://github.com/solana-labs/solana-program-library