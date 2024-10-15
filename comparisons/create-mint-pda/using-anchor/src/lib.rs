use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

declare_id!("Examp1eCreateMintPdaUsingAnchor1111111111111");

#[program]
pub mod create_mint_pda_using_anchor {
    use super::*;

    #[allow(unused_variables)]
    pub fn init_mint(
        _ctx: Context<InitMint>,
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8, mint_authority: Pubkey, freeze_authority: Option<Pubkey>)]
pub struct InitMint<'a> {
    #[account(mut)]
    payer: Signer<'a>,

    token_program: Interface<'a, TokenInterface>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority,
        mint::freeze_authority = freeze_authority.unwrap_or(mint_authority),
        // extensions::close_authority::authority = arbitrary_authority,
        seeds = [b"mint"],
        bump,
    )]
    new_mint: InterfaceAccount<'a, Mint>,

    system_program: Program<'a, System>,
}
