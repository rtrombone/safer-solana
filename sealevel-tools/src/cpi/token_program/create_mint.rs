use core::mem::size_of;

use crate::{
    account_info::{is_any_token_program_id, Account},
    cpi::{system_program::CreateAccount, CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    spl_token_2022::state::Mint,
};

use super::EMPTY_EXTENSION_LEN;

/// Arguments to create a mint account. This method creates an account for one of the Token programs
/// and initializes it as a mint.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, TokenProgram,
///         EnumeratedAccountConstraints, Account, Signer,
///     },
///     cpi::token_program::{CreateMint, InitializeMetadataPointerData, InitializeMintExtensions},
///     entrypoint::{NoStdAccountInfo, ProgramResult},
///     pubkey::Pubkey,
/// };
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[NoStdAccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must writable signer (A.K.A. our payer).
///     let (_, payer) =
///         try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     let (new_mint_addr, new_mint_bump) =
///         Pubkey::find_program_address(&[b"mint"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_mint_account) = try_next_enumerated_account::<Account<true>>(
///         &mut accounts_iter,
///         EnumeratedAccountConstraints {
///             key: Some(&new_mint_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account is the mint authority.
///     let (_, mint_authority) =
///         try_next_enumerated_account_info(&mut accounts_iter, Default::default())?;
///
///     // Next account is which token program to use.
///     let (_, token_program) =
///         try_next_enumerated_account::<TokenProgram>(&mut accounts_iter, Default::default())?;
///
///     CreateMint {
///         token_program_id: token_program.key(),
///         payer: payer.as_cpi_authority(),
///         mint: new_mint_account.as_cpi_authority(Some(&[b"mint", &[new_mint_bump]])),
///         mint_authority: mint_authority.key(),
///         decimals: 9,
///         freeze_authority: None,
///         extensions: InitializeMintExtensions {
///             metadata_pointer: Some(InitializeMetadataPointerData {
///                 authority: Some(mint_authority.key()),
///                 metadata: new_mint_account.key(),
///             }),
///             non_transferable: true,
///             ..Default::default()
///         }
///     }
///     .try_into_invoke()?;
///
///     Ok(())
/// }
/// ```
pub struct CreateMint<'a, 'b: 'a> {
    pub token_program_id: &'a Pubkey,
    pub payer: CpiAuthority<'a, 'b>,
    pub mint: CpiAuthority<'a, 'b>,
    pub mint_authority: &'a Pubkey,
    pub decimals: u8,
    pub freeze_authority: Option<&'a Pubkey>,
    pub extensions: InitializeMintExtensions<'a>,
}

/// Optional extensions for mint accounts. These extensions are used to add additional features to
/// mint accounts.
#[derive(Default)]
pub struct InitializeMintExtensions<'a> {
    pub close_authority: Option<&'a Pubkey>,
    pub group_pointer: Option<InitializeGroupPointerData<'a>>,
    pub group_member_pointer: Option<InitializeGroupMemberPointerData<'a>>,
    pub metadata_pointer: Option<InitializeMetadataPointerData<'a>>,
    pub non_transferable: bool,
    pub permanent_delegate: Option<&'a Pubkey>,
    pub transfer_fee_config: Option<InitializeTransferFeeConfigData<'a>>,
    pub transfer_hook: Option<InitializeTransferHookData<'a>>,

    /// If specified, [Self::transfer_fee_config] cannot be specified unless
    /// [Self::confidential_transfer_fee_config] is also specified.
    pub confidential_transfer: Option<InitializeConfidentialTransferData<'a>>,

    /// If specified, [Self::transfer_fee_config] and [Self::confidential_transfer] must also be
    /// specified.
    pub confidential_transfer_fee_config: Option<InitializeConfidentialTransferFeeConfigData<'a>>,
    // FIXME: Uncomment when the extension is implemented.
    //pub confidential_mint_burn: Option<InitializeConfidentialMintBurnData<'a>>,
}
/// Data required to initialize the group pointer extension, which is used to establish a collection
/// of mints.
pub struct InitializeGroupPointerData<'a> {
    /// Who has authority to update the group. If [None], the group cannot be modified.
    pub authority: Option<&'a Pubkey>,

    /// Where the group is stored.
    ///
    /// ### Notes
    ///
    /// The initialize group pointer instruction actually takes an optional group account, where if
    /// both the authority and group pubkeys are [None], then the Token program will revert because
    /// at least one of these values must be present. So if you want to initialize the group
    /// pointer, this data must be specified.
    pub group: &'a Pubkey,
}

/// Data required to initialize the group member pointer extension, which is used to associate a
/// mint with a group (via group pointer extension).
pub struct InitializeGroupMemberPointerData<'a> {
    /// Who has authority to update the group member. If [None], the group member cannot be
    /// modified.
    pub authority: Option<&'a Pubkey>,

    /// Where the group member is stored.
    ///
    /// ### Notes
    ///
    /// The initialize group member pointer instruction actually takes an optional member account,
    /// where if both the authority and member pubkeys are [None], then the Token program will
    /// revert because at least one of these values must be present. So if you want to initialize
    /// the group member pointer, this data must be specified.
    pub group_member: &'a Pubkey,
}

/// Data required to initialize the metadata pointer extension, which is used to add descriptive
/// information to a mint.
pub struct InitializeMetadataPointerData<'a> {
    /// Who has authority to update metadata. If [None], the metadata cannot be modified.
    pub authority: Option<&'a Pubkey>,

    /// Where the metadata is stored.
    ///
    /// ### Notes
    ///
    /// The initialize metadata pointer instruction actually takes an optional metadata account,
    /// where if both the authority and metadata pubkeys are [None], then the Token program will
    /// revert because at least one of these values must be present. So if you want to initialize
    /// the metadata pointer, this data must be specified.
    pub metadata: &'a Pubkey,
}

/// Data required to initialize the transfer fee extension, which is used to charge a fee for
/// transferring tokens.
pub struct InitializeTransferFeeConfigData<'a> {
    /// Who has authority to update the config. If [None], the config cannot be modified.
    pub config_authority: Option<&'a Pubkey>,

    /// Who has authority to withhold fees. If [None], no one can withdraw fees.
    pub withdraw_withheld_authority: Option<&'a Pubkey>,

    /// The basis points for the fee. Cannot exceed
    /// [spl_token_2022::extension::transfer_fee::MAX_FEE_BASIS_POINTS].
    pub basis_points: u16,

    /// The maximum fee.
    pub maximum_fee: u64,
}

/// Data required to initialize the transfer hook extension, which is used to execute custom logic
/// when transferring tokens.
pub struct InitializeTransferHookData<'a> {
    /// Who has authority to update the program ID. If [None], the program ID cannot be modified.
    pub authority: Option<&'a Pubkey>,

    /// Program for transfer hook CPI.
    ///
    /// ### Notes
    ///
    /// The initialize transfer hook  instruction actually takes an optional program ID, where if
    /// both the authority and program ID pubkeys are [None], then the Token program will revert
    /// because at least one of these values must be present. So if you want to initialize the
    /// transfer hook extension, this data must be specified.
    pub program_id: &'a Pubkey,
}

/// Data required to initialize the confidential transfer extension, which is used to hide the
/// transfer amount.
pub struct InitializeConfidentialTransferData<'a> {
    /// Authority to modify the `ConfidentialTransferMint` configuration and to approve new
    /// accounts.
    pub authority: Option<&'a Pubkey>,

    /// Determines if newly configured accounts must be approved by [Self::authority] before they
    /// may be used by the user.
    pub auto_approve_new_accounts: bool,

    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal: Option<&'a [u8; 32]>,
}

/// Data required to initialize the confidential transfer fee extension, which is used to hide the
/// transfer fee amount.
pub struct InitializeConfidentialTransferFeeConfigData<'a> {
    /// Optional authority to set the withdraw withheld authority ElGamal key.
    pub authority: Option<&'a Pubkey>,

    /// Withheld fees from accounts must be encrypted with this ElGamal key.
    ///
    /// Note that whoever holds the ElGamal private key for this ElGamal public key has the ability
    /// to decode any withheld fee amount that are associated with accounts. When combined with the
    /// fee parameters, the withheld fee amounts can reveal information about transfer amounts.
    pub withdraw_withheld_authority_elgamal: &'a [u8; 32],
}

// FIXME: Uncomment when the extension is implemented.
// /// Data required to initialize the confidential mint burn extension, which is used to hide the
// /// supply amount.
// pub struct InitializeConfidentialMintBurnData<'a> {
//     /// The ElGamal pubkey used to encrypt the confidential supply.
//     pub supply_elgamal: &'a [u8; 32],

//     /// The initial 0 supply encrypted with the supply aes key.
//     pub decryptable_supply: &'a [u8; 36],
// }

const ONLY_AUTHORITY_LEN: usize = {
    EMPTY_EXTENSION_LEN // type + length
    + size_of::<Pubkey>() // authority
};

const AUTHORITY_POINTER_LEN: usize = {
    ONLY_AUTHORITY_LEN // type + length + authority
    + size_of::<Pubkey>() // pointer
};

impl<'a, 'b: 'a> CreateMint<'a, 'b> {
    /// Try to consume arguments to perform CPI calls.
    ///
    /// ### Notes
    ///
    /// Extension CPI calls are optional and will only be invoked when [Some] is provided for any
    /// of the optional extension arguments. See [InitializeMintExtensions] for more information.
    #[inline(always)]
    pub fn try_into_invoke(self) -> Result<Account<'b, true>, ProgramError> {
        let Self {
            token_program_id,
            payer,
            mint,
            mint_authority,
            decimals,
            freeze_authority,
            extensions:
                InitializeMintExtensions {
                    close_authority,
                    group_pointer,
                    group_member_pointer,
                    metadata_pointer,
                    non_transferable,
                    permanent_delegate,
                    transfer_fee_config,
                    transfer_hook,
                    confidential_transfer,
                    confidential_transfer_fee_config,
                    // FIXME: Uncomment when the extension is implemented.
                    // confidential_mint_burn,
                },
        } = self;

        let add_close_authority = close_authority.is_some();
        let add_group_pointer = group_pointer.is_some();
        let add_group_member_pointer = group_member_pointer.is_some();
        let add_metadata_pointer = metadata_pointer.is_some();
        let add_permanent_delegate = permanent_delegate.is_some();
        let add_transfer_fee_config = transfer_fee_config.is_some();
        let add_transfer_hook = transfer_hook.is_some();
        let add_confidential_transfer = confidential_transfer.is_some();
        let add_confidential_transfer_fee_config = confidential_transfer_fee_config.is_some();
        // FIXME: Uncomment when the extension is implemented.
        // let add_confidential_mint_burn = confidential_mint_burn.is_some();

        let mint_account = if add_close_authority
            || add_group_pointer
            || add_group_member_pointer
            || add_metadata_pointer
            || non_transferable
            || add_permanent_delegate
            || add_transfer_fee_config
            || add_transfer_hook
            || add_confidential_transfer
            || add_confidential_transfer_fee_config
        // FIXME: Uncomment when the extension is implemented.
        // || add_confidential_mint_burn
        {
            if token_program_id != &spl_token_2022::ID {
                return Err(super::ERROR_EXTENSIONS_UNSUPPORTED.into());
            }

            // Add to this depending on which extensions to add to the mint.
            let mut total_space = super::BASE_WITH_EXTENSIONS_LEN;

            if add_close_authority {
                total_space += ONLY_AUTHORITY_LEN;
            }
            if add_group_pointer {
                total_space += AUTHORITY_POINTER_LEN;
            }
            if add_group_member_pointer {
                total_space += AUTHORITY_POINTER_LEN;
            }
            if add_metadata_pointer {
                total_space += AUTHORITY_POINTER_LEN;
            }
            if non_transferable {
                total_space += EMPTY_EXTENSION_LEN;
            }
            if add_permanent_delegate {
                total_space += ONLY_AUTHORITY_LEN;
            }
            if add_transfer_fee_config {
                total_space += {
                    EMPTY_EXTENSION_LEN // type + length
                    + size_of::<Pubkey>() // authority
                    + size_of::<Pubkey>() // withdraw_withheld_authority
                    + size_of::<u64>() // withheld_amount
                    + size_of::<u64>() // older_transfer_fee_epoch
                    + size_of::<u64>() // older_transfer_fee_maximum_fee
                    + size_of::<u16>() // older_basis_points
                    + size_of::<u64>() // newer_transfer_fee_epoch
                    + size_of::<u64>() // newer_transfer_fee_maximum_fee
                    + size_of::<u16>() // newer_basis_points
                };
            }
            if add_transfer_hook {
                total_space += AUTHORITY_POINTER_LEN;
            }
            if add_confidential_transfer {
                total_space += {
                    EMPTY_EXTENSION_LEN // type + length
                    + size_of::<Pubkey>() // authority
                    + size_of::<u8>() // auto_approve_new_accounts
                    + size_of::<Pubkey>() // auditor_elgamal
                };
            }
            if add_confidential_transfer_fee_config {
                total_space += {
                    EMPTY_EXTENSION_LEN // type + length
                    + size_of::<Pubkey>() // authority
                    + size_of::<[u8; 32]>() // withdraw_withheld_authority_elgamal
                    + size_of::<bool>() // harvest_to_mint_enabled
                    + 64 // withheld_amount (encrypted)
                };
            }
            // FIXME: Uncomment when the extension is implemented.
            // if add_confidential_mint_burn {
            //     total_space += {
            //         EMPTY_EXTENSION_LEN // type + length
            //         + 64 // confidential_supply
            //         + 36 // decryptable_supply
            //         + 32 // supply_elgamal
            //     };
            // }

            // First create the mint account by assigning it to the token program.
            let mint_account = CreateAccount {
                payer,
                to: mint,
                program_id: token_program_id,
                space: Some(total_space),
                lamports: None,
            }
            .try_into_invoke()?;

            if let Some(close_authority) = close_authority {
                super::extensions::InitializeMintCloseAuthority {
                    token_program_id,
                    mint: &mint_account,
                    authority: Some(close_authority),
                }
                .into_invoke();
            }

            if let Some(InitializeGroupPointerData { authority, group }) = group_pointer {
                super::extensions::InitializeGroupPointer {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    group: Some(group),
                }
                .into_invoke();
            }

            if let Some(InitializeGroupMemberPointerData {
                authority,
                group_member,
            }) = group_member_pointer
            {
                super::extensions::InitializeGroupMemberPointer {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    group_member: Some(group_member),
                }
                .into_invoke();
            }

            if let Some(InitializeMetadataPointerData {
                authority,
                metadata,
            }) = metadata_pointer
            {
                super::extensions::InitializeMetadataPointer {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    metadata: Some(metadata),
                }
                .into_invoke();
            }

            if non_transferable {
                super::extensions::InitializeNonTransferable {
                    token_program_id,
                    mint: &mint_account,
                }
                .into_invoke();
            }

            if let Some(delegate) = permanent_delegate {
                super::extensions::InitializePermanentDelegate {
                    token_program_id,
                    mint: &mint_account,
                    delegate,
                }
                .into_invoke();
            }

            if let Some(InitializeTransferFeeConfigData {
                config_authority,
                withdraw_withheld_authority,
                basis_points,
                maximum_fee,
            }) = transfer_fee_config
            {
                super::extensions::InitializeTransferFeeConfig {
                    token_program_id,
                    mint: &mint_account,
                    config_authority,
                    withdraw_withheld_authority,
                    basis_points,
                    maximum_fee,
                }
                .into_invoke();
            }

            if let Some(InitializeTransferHookData {
                authority,
                program_id,
            }) = transfer_hook
            {
                super::extensions::InitializeTransferHook {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    program_id: Some(program_id),
                }
                .into_invoke();
            }

            if let Some(InitializeConfidentialTransferData {
                authority,
                auto_approve_new_accounts,
                auditor_elgamal,
            }) = confidential_transfer
            {
                super::extensions::InitializeConfidentialTransfer {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    auto_approve_new_accounts,
                    auditor_elgamal,
                }
                .into_invoke();
            }

            if let Some(InitializeConfidentialTransferFeeConfigData {
                authority,
                withdraw_withheld_authority_elgamal,
            }) = confidential_transfer_fee_config
            {
                super::extensions::InitializeConfidentialTransferFeeConfig {
                    token_program_id,
                    mint: &mint_account,
                    authority,
                    withdraw_withheld_authority_elgamal,
                }
                .into_invoke();
            }

            // FIXME: Uncomment when the extension is implemented.
            // if let Some(InitializeConfidentialMintBurnData {
            //     supply_elgamal,
            //     decryptable_supply,
            // }) = confidential_mint_burn
            // {
            //     super::extensions::InitializeConfidentialMintBurn {
            //         token_program_id,
            //         mint: &mint_account,
            //         supply_elgamal,
            //         decryptable_supply,
            //     }
            //     .into_invoke();
            // }

            mint_account
        } else {
            if !is_any_token_program_id(token_program_id) {
                return Err(super::ERROR_EXPECTED_TOKEN_PROGRAM.into());
            }

            // First create the mint account by assigning it to the token program.
            CreateAccount {
                payer,
                to: mint,
                program_id: token_program_id,
                space: Some(Mint::LEN),
                lamports: None,
            }
            .try_into_invoke()?
        };

        _invoke_initialize_mint2(
            token_program_id,
            &mint_account,
            mint_authority,
            freeze_authority,
            decimals,
        );

        Ok(mint_account)
    }
}

/// Arguments for the initialize mint instruction (version 2), which initializes a mint account for
/// one of the Token programs. Only use this instruction if you have already created the mint
/// account via the System program.
///
/// ### Notes
///
/// It is preferred to use [CreateMint] instead of initializing a mint by itself because the other
/// method will create the account and initialize it as a mint in one action.
pub struct InitializeMint<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub mint_authority: &'a Pubkey,
    pub freeze_authority: Option<&'a Pubkey>,
    pub decimals: u8,
}

impl<'a> InitializeMint<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            mint_authority,
            freeze_authority,
            decimals,
        } = self;

        _invoke_initialize_mint2(
            token_program_id,
            mint,
            mint_authority,
            freeze_authority,
            decimals,
        );
    }
}

#[inline(always)]
fn _invoke_initialize_mint2(
    token_program_id: &Pubkey,
    mint: &NoStdAccountInfo,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) {
    const IX_DATA_LEN: usize = {
        size_of::<u8>() // selector
        + size_of::<u8>() // decimals
        + size_of::<Pubkey>() // mint_authority
        + size_of::<u8>() + size_of::<Pubkey>() // freeze_authority
    };

    let mut instruction_data = [0; IX_DATA_LEN];

    // Initialize mint 2 selector == 20.
    instruction_data[0] = 20;
    instruction_data[1] = decimals;
    instruction_data[2..34].copy_from_slice(&mint_authority.to_bytes());

    if let Some(freeze_authority) = freeze_authority {
        instruction_data[34] = 1;
        instruction_data[35..67].copy_from_slice(&freeze_authority.to_bytes());
    }

    CpiInstruction {
        program_id: token_program_id,
        accounts: &[mint.to_meta_c()],
        data: &instruction_data,
    }
    .invoke_signed(&[mint.to_info_c()], &[]);
}
