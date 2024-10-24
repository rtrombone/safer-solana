use solana_program::{
    account_info::AccountInfo, program::invoke, program_error::ProgramError, program_pack::Pack,
};
use spl_token_2022::instruction::initialize_account3;

use crate::{
    account_info::DataAccount,
    cpi::system_program::{try_create_account, CreateAccount},
    error::SealevelToolsError,
    types::{InputAccount, InputAuthority},
};

pub struct CreateTokenAccount<'a, 'b, 'c> {
    pub payer: InputAuthority<'a, 'b, 'c>,
    pub token_account: InputAuthority<'a, 'b, 'c>,
    pub mint: InputAccount<'a, 'c>,
    pub token_account_owner: InputAccount<'a, 'c>,
    pub account_infos: &'c [AccountInfo<'a>],
}

pub fn try_create_token_account<'a, 'c>(
    CreateTokenAccount {
        payer,
        token_account,
        mint,
        token_account_owner,
        account_infos,
    }: CreateTokenAccount<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    // Determine mint account info. This info is used to find the token program ID.
    let mint_info = match &mint {
        InputAccount::Key(mint_key) => account_infos
            .iter()
            .find(|info| info.key == *mint_key)
            .ok_or_else(|| {
                SealevelToolsError::Cpi("token_program", format!("Cannot find mint {mint_key}"))
            })?,
        InputAccount::Info(info) => info,
    };

    // Create the token account by assigning it to the token program.
    let token_account = try_create_account(CreateAccount {
        payer,
        to: token_account,
        space: spl_token_2022::state::Account::LEN as u64,
        program_id: mint_info.owner,
        account_infos,
    })?;

    // NOTE: Token program ID passed into this method is checked against either token program IDs.
    // This method will revert with IncorrectProgramId if the token program ID is not correct.
    let instruction = initialize_account3(
        mint_info.owner,
        token_account.key,
        mint_info.key,
        token_account_owner.key(),
    )?;

    invoke(&instruction, account_infos)?;

    Ok(token_account)
}
