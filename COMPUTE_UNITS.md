# Compute Units (CU)

Optimize CU usage in your Solana programs! Maximum compute per block is
[48 million CU].

When sending transactions, invoking the Compute Budget program to set the
compute units for your transaction to as low as possible will make it more
likely for your transaction to make it into the next slot. And with priority
fees, the total cost of landing your transaction will be optimized (although
cost-per-unit is quite small).

Some optimizations can come with a trade-off on safety (e.g. choosing when to
derive a program address), so be aware of what you sacrifice when you make a CU
optimization.

If you are using the Anchor framework, this [Solana article] gives a very basic
overview of CU usage with examples relating to this framework. But also, if you
are using the Anchor framework, you may not care too much about CU optimization.

## Helpful Information

- [How does the Solana BPF VM calculate how many Compute Units a program consumes?]
  - See [relevant CU config]. Example costs:
    - Creating a program address with a known bump costs 1,500 CU.
    - Deriving a program address costs 1,500 CU per bump iteration (e.g. bump ==
      253 means 4,500 CU because it requires 3 iterations including checking
      bump == 255).
    - Performing a CPI call starts at 1,000 CU.
    - Logging remaining CU is 100 CU (so if you perform
      `solana_program::log::sol_log_compute_units()`, subtract 100 from any CU
      difference you calculate using these logs).
- [Priority Fees: Understanding Solana's Transaction Fee Mechanics]
- [Sending Transactions]

[48 million CU]: https://github.com/anza-xyz/agave/blob/d5a84daebd2a7225684aa3f722b330e9d5381e76/cost-model/src/block_cost_limits.rs#L68
[How does the Solana BPF VM calculate how many Compute Units a program consumes?]: https://solana.stackexchange.com/questions/15119/how-does-the-solana-bpf-vm-calculate-how-many-compute-units-a-program-consumes
[Priority Fees: Understanding Solana's Transaction Fee Mechanics]: https://www.helius.dev/blog/priority-fees-understanding-solanas-transaction-fee-mechanics
[Sending Transactions]: https://docs.triton.one/chains/solana/sending-txs
[Solana article]: https://solana.com/developers/guides/advanced/how-to-optimize-compute
[relevant CU config]: https://github.com/anza-xyz/agave/blob/d5a84daebd2a7225684aa3f722b330e9d5381e76/compute-budget/src/compute_budget.rs#L145-L192