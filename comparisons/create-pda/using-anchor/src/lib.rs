use anchor_lang::prelude::*;

declare_id!("Examp1eCreateAccountUsingAnchor1111111111111");

#[program]
pub mod create_pda_using_anchor {
    use super::*;

    pub fn init_thing(ctx: Context<InitThing>) -> Result<()> {
        ctx.accounts.new_thing.set_inner(Thing { data: 69 });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitThing<'a> {
    #[account(mut)]
    payer: Signer<'a>,

    #[account(
        init,
        payer = payer,
        space = 16,
        seeds = [b"thing"],
        bump,
    )]
    new_thing: Account<'a, Thing>,

    system_program: Program<'a, System>,
}

#[account]
#[derive(Debug, PartialEq)]
pub struct Thing {
    pub data: u64,
}
