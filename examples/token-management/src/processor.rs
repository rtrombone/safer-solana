use sealevel_tools::{
    account_info::{
        try_next_enumerated_account, AccountInfoConstraints, Authority, Payer, ReadonlyAccount,
        SystemProgram, TokenProgram, WritableAccount, WritableTokenProgramAccount,
    },
    cpi::{
        set_return_data,
        token_program::{self as token_program_cpi},
    },
    entrypoint::{NoStdAccountInfo, ProgramResult},
    pubkey::Pubkey,
};

use crate::{
    instruction::{ExtensionTypes, InitMintWithExtensionsData},
    state,
};

#[inline(always)]
pub fn init_ata(accounts: &[NoStdAccountInfo], idempotent: bool) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) = try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;

    // Second account is the new token account. It may be safer to verify the ATA address for this
    // account. But the create instruction should fail if the account key is incorrect.
    let (_, new_ata) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the owner of the new ATA.
    let (_, owner) =
        try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;

    // Fourth account is the mint. Disregard checking the mint PDA (but in a real program, you
    // probably should check). We don't care to deserialize the mint account.
    let (_, mint_account) =
        try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;

    // Fifth account is the System program to create the new account.
    let (_, system_program) =
        try_next_enumerated_account::<SystemProgram>(&mut accounts_iter, Default::default())?;

    // Sixth account is which token program to use to initialize the token account.
    let (_, token_program) =
        try_next_enumerated_account::<TokenProgram>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    sealevel_tools::cpi::ata_program::Create {
        ata_program_id: None,
        payer: payer.as_cpi_authority(),
        associated_account: &new_ata,
        account_owner: &owner,
        mint: &mint_account,
        system_program: &system_program,
        token_program: &token_program,
        idempotent,
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn init_mint_with_extensions(
    accounts: &[NoStdAccountInfo],
    InitMintWithExtensionsData {
        decimals,
        freeze_authority,
        close_authority,
        group_pointer,
        group_member_pointer,
        metadata_pointer,
        non_transferable,
        permanent_delegate,
        transfer_fee,
        transfer_hook,
        confidential_transfer,
        confidential_transfer_fee,
    }: InitMintWithExtensionsData,
) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) = try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;

    let (new_mint_addr, new_mint_bump) = state::find_mint_address();

    // Second account is which token program to use.
    let (_, token_program) =
        try_next_enumerated_account::<TokenProgram>(&mut accounts_iter, Default::default())?;

    // Third account is the new mint.
    let (_, new_mint_account) = try_next_enumerated_account::<WritableAccount>(
        &mut accounts_iter,
        AccountInfoConstraints {
            key: Some(&new_mint_addr),
            ..Default::default()
        },
    )?;

    let (mint_authority_addr, _) = state::find_authority_address();

    // solana_program::log::sol_log_compute_units();

    let extensions = token_program_cpi::InitializeMintExtensions {
        close_authority: if close_authority {
            Some(&mint_authority_addr)
        } else {
            None
        },
        group_pointer: if group_pointer {
            Some(token_program_cpi::InitializeGroupPointerData {
                authority: Some(&mint_authority_addr),
                group: &new_mint_addr,
            })
        } else {
            None
        },
        group_member_pointer: if group_member_pointer {
            Some(token_program_cpi::InitializeGroupMemberPointerData {
                authority: Some(&mint_authority_addr),
                group_member: &new_mint_addr,
            })
        } else {
            None
        },
        metadata_pointer: if metadata_pointer {
            Some(token_program_cpi::InitializeMetadataPointerData {
                authority: Some(&mint_authority_addr),
                metadata: &new_mint_addr,
            })
        } else {
            None
        },
        non_transferable,
        permanent_delegate: if permanent_delegate {
            Some(&mint_authority_addr)
        } else {
            None
        },
        transfer_fee_config: if transfer_fee {
            Some(token_program_cpi::InitializeTransferFeeConfigData {
                config_authority: Some(&mint_authority_addr),
                withdraw_withheld_authority: Some(&mint_authority_addr),
                basis_points: 2,
                maximum_fee: 1_000,
            })
        } else {
            None
        },
        transfer_hook: if transfer_hook {
            Some(token_program_cpi::InitializeTransferHookData {
                authority: Some(&mint_authority_addr),
                program_id: &crate::ID,
            })
        } else {
            None
        },
        confidential_transfer: if confidential_transfer {
            Some(token_program_cpi::InitializeConfidentialTransferData {
                authority: Some(&mint_authority_addr),
                auto_approve_new_accounts: true,
                auditor_elgamal: Some(&[1; 32]),
            })
        } else {
            None
        },
        confidential_transfer_fee_config: if confidential_transfer_fee {
            Some(
                token_program_cpi::InitializeConfidentialTransferFeeConfigData {
                    authority: Some(&mint_authority_addr),
                    withdraw_withheld_authority_elgamal: &[2; 32],
                },
            )
        } else {
            None
        },
    };

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::CreateMint {
        token_program_id: token_program.key(),
        payer: payer.as_cpi_authority(),
        mint: new_mint_account.as_cpi_authority(Some(&[state::MINT_SEED, &[new_mint_bump]])),
        mint_authority: &mint_authority_addr,
        decimals,
        freeze_authority: freeze_authority.as_ref(),
        extensions,
    }
    .try_into_invoke()?;

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn init_token_account(
    accounts: &[NoStdAccountInfo],
    owner: Pubkey,
    immutable_owner: bool,
) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let (_, payer) = try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;

    // Second account is the mint.
    let (_, mint_account) = try_next_enumerated_account::<ReadonlyAccount>(
        &mut accounts_iter,
        AccountInfoConstraints {
            key: Some(&state::find_mint_address().0),
            ..Default::default()
        },
    )?;

    let (new_token_account_addr, new_token_account_bump) =
        state::find_token_account_address(&owner);

    // Third account is the new token account.
    let (_, new_token_account) = try_next_enumerated_account::<WritableAccount>(
        &mut accounts_iter,
        AccountInfoConstraints {
            key: Some(&new_token_account_addr),
            ..Default::default()
        },
    )?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::CreateTokenAccount {
        payer: payer.as_cpi_authority(),
        token_account: new_token_account.as_cpi_authority(Some(&[
            state::TOKEN_SEED,
            owner.as_ref(),
            &[new_token_account_bump],
        ])),
        mint: &mint_account,
        token_account_owner: &owner,
        immutable_owner,
    }
    .try_into_invoke()?;

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn mint_to(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the mint. Disregard checking the mint PDA (but in a real program, you
    // probably should check). We don't care to deserialize the mint account.
    let (_, mint_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = mint_account.owner();

    // Second account is the destination token account. We don't care to deserialize the token
    // account. No need to check whether this account belongs to a Token program because we enforce
    // the Token program ID from the mint account.
    let (_, destination_account) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the mint authority.
    //
    // Since we need the bump for the mint authority's signer seeds, we will find the mint
    // authority's address. But if this bump were cached, we could disregard the pubkey check since
    // the signer seeds would be "wrong" for any account that is not the actual mint authority.
    let (mint_authority_addr, mint_authority_bump) = state::find_authority_address();

    let (_, mint_authority) = try_next_enumerated_account::<ReadonlyAccount>(
        &mut accounts_iter,
        AccountInfoConstraints {
            key: Some(&mint_authority_addr),
            ..Default::default()
        },
    )?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::MintTo {
        token_program_id,
        mint: &mint_account,
        destination: &destination_account,
        mint_authority: mint_authority
            .as_cpi_authority(Some(&[state::AUTHORITY_SEED, &[mint_authority_bump]])),
        amount,
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn suboptimal_mint_to(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the mint. Disregard checking the mint PDA (but in a real program, you
    // probably should check). We don't care to deserialize the mint account.
    let (_, mint_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = mint_account.owner();

    // Second account is the destination token account. We don't care to deserialize the token
    // account. No need to check whether this account belongs to a Token program because we enforce
    // the Token program ID from the source account.
    let (_, destination_account) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the mint authority.
    //
    // Since we need the bump for the mint authority's signer seeds, we will find the mint
    // authority's address. But if this bump were cached, we could disregard the pubkey check since
    // the signer seeds would be "wrong" for any account that is not the actual mint authority.
    let (mint_authority_addr, mint_authority_bump) = state::find_authority_address();

    let (_, mint_authority) = try_next_enumerated_account::<ReadonlyAccount>(
        &mut accounts_iter,
        AccountInfoConstraints {
            key: Some(&mint_authority_addr),
            ..Default::default()
        },
    )?;

    // solana_program::log::sol_log_compute_units();

    sealevel_tools::cpi::invoke_signed(
        &spl_token_2022::instruction::mint_to(
            token_program_id,
            mint_account.key(),
            destination_account.key(),
            mint_authority.key(),
            &[],
            amount,
        )?,
        &[
            mint_account.to_info_c(),
            destination_account.to_info_c(),
            mint_authority.to_info_c(),
        ],
        &[&[state::AUTHORITY_SEED, &[mint_authority_bump]]],
    );

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn burn(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the source token account. We don't care to deserialize the token account.
    let (_, source_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = source_account.owner();

    // Second account is the mint. No need to check whether this account belongs to a Token program
    // because we enforce the Token program ID from the source account.
    let (_, mint_account) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the authority, which should have been delegated by the owner of the source
    // account.
    let (_, authority) =
        try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::Burn {
        token_program_id,
        source: &source_account,
        mint: &mint_account,
        authority: authority.as_cpi_authority(),
        amount,
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn transfer(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the source token account. We don't care to deserialize the token account.
    let (_, source_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = source_account.owner();

    // Second account is the destination token account. No need to check whether this account
    // belongs to a Token program because we enforce the Token program ID from the source account.
    let (_, destination_account) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the authority, which should have been delegated by the owner of the source
    // account.
    let (_, authority) =
        try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::Transfer {
        token_program_id,
        source: &source_account,
        destination: &destination_account,
        authority: authority.as_cpi_authority(),
        amount,
        checked: None,
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn transfer_checked(accounts: &[NoStdAccountInfo], amount: u64, decimals: u8) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the source token account. We don't care to deserialize the token account.
    let (_, source_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = source_account.owner();

    // Second account is the mint. No need to check whether this account belongs to a Token program
    // because we enforce the Token program ID from the source account.
    let (_, mint_account) =
        try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the destination token account. No need to check whether this account
    // belongs to a Token program because we enforce the Token program ID from the source account.
    let (_, destination_account) =
        try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;

    // Fourth account is the authority, which should have been delegated by the owner of the source
    // account.
    let (index, authority) =
        try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::Transfer {
        token_program_id,
        source: &source_account,
        destination: &destination_account,
        authority: authority.as_cpi_authority(),
        amount,
        checked: Some(token_program_cpi::UseTransferChecked {
            mint: &mint_account,
            decimals,
            additional_accounts: Some(&accounts[index..]),
        }),
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn approve(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the source token account. We don't care to deserialize the token account.
    let (_, source_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = source_account.owner();

    // Second account is the destination token account. No need to check whether this account
    // belongs to a Token program because we enforce the Token program ID from the source account.
    let (_, delegate) =
        try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;

    // Third account is the authority, which should be the owner of the source account.
    let (_, authority) =
        try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::Approve {
        token_program_id,
        source: &source_account,
        delegate: &delegate,
        authority: authority.as_cpi_authority(),
        amount,
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn revoke(accounts: &[NoStdAccountInfo]) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the source token account. We don't care to deserialize the token account.
    let (_, source_account) = try_next_enumerated_account::<WritableTokenProgramAccount>(
        &mut accounts_iter,
        Default::default(),
    )?;

    let token_program_id = source_account.owner();

    // Second account is the authority, which should be the owner of the source account.
    let (_, authority) =
        try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    token_program_cpi::Revoke {
        token_program_id,
        source: &source_account,
        authority: authority.as_cpi_authority(),
    }
    .into_invoke();

    // solana_program::log::sol_log_compute_units();

    Ok(())
}

#[inline(always)]
pub fn get_account_data_size(
    accounts: &[NoStdAccountInfo],
    extension_types: ExtensionTypes,
) -> ProgramResult {
    // solana_program::log::sol_log_compute_units();

    let mut accounts_iter = accounts.iter().enumerate();

    // First account is the mint. We don't care to deserialize the token account.
    let (_, mint_account) =
        try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;

    // solana_program::log::sol_log_compute_units();

    let account_size = token_program_cpi::GetAccountDataSize {
        token_program_id: mint_account.owner(),
        mint: &mint_account,
        extensions: &extension_types.0,
    }
    .into_return_data();

    // solana_program::log::sol_log_compute_units();

    set_return_data(&account_size.to_le_bytes());

    // solana_program::log::sol_log_compute_units();

    Ok(())
}
