use example_token_management::{
    instruction::{ExtensionTypes, InitMintWithExtensionsData, ProgramInstruction},
    state, ID,
};
use examples_common::{is_compute_units_within, is_program_failure, TestResult, TestSuccess};
use sealevel_tools::account::{legacy_token, token_extensions, AssociatedTokenAccountSeeds};
use solana_program_test::{tokio, BanksClient, ProgramTest};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
    transaction_context::TransactionReturnData,
};
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensionsOwned},
    state::{Account, Mint},
};

const CU_TOLERANCE: u64 = 50;
const DEFAULT_OWNER: Pubkey = solana_sdk::pubkey!("Defau1towner1111111111111111111111111111111");

#[tokio::test]
async fn test_init_mint_token_program() {
    let decimals = 9;

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        legacy_token::ID,
        decimals,
        None, // freeze_authority
        None, // extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_350,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_token_account_token_program() {
    let owner = DEFAULT_OWNER;
    let immutable_owner = false;

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        legacy_token::ID,
        owner,
        immutable_owner,
        None, // mint_extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_950,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_ata_token_program() {
    let owner = DEFAULT_OWNER;

    let TestSuccess { tx_meta, .. } = InitAtaTest::set_up(
        legacy_token::ID,
        owner,
        false, // idempotent
        None,  // mint_extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        24_400,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitAtaTest::set_up(
        legacy_token::ID,
        owner,
        true, // idempotent
        None, // mint_extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        24_400,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_get_account_data_size_token_program() {
    let extensions = [ExtensionType::ImmutableOwner];

    let TestSuccess { tx_meta, .. } = GetAccountDataSizeTest::set_up(legacy_token::ID, &extensions)
        .await
        .run()
        .await
        .success()
        .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        3_230,
        CU_TOLERANCE
    ));
    assert_eq!(
        tx_meta.return_data,
        Some(TransactionReturnData {
            program_id: ID,
            data: 165_u64.to_le_bytes().to_vec(),
        })
    );
}

#[tokio::test]
async fn test_mint_to_token_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = true;

    let TestSuccess { tx_meta, .. } =
        MintToTest::set_up(legacy_token::ID, destination_owner, amount, optimized)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_090,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_suboptimal_mint_to_token_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = false;

    let TestSuccess { tx_meta, .. } =
        MintToTest::set_up(legacy_token::ID, destination_owner, amount, optimized)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_550,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_burn_token_program() {
    let source_owner = Keypair::new();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } = BurnTest::set_up(legacy_token::ID, &source_owner, amount)
        .await
        .run()
        .await
        .success()
        .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        6_230,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_transfer_token_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        TransferTest::set_up(legacy_token::ID, &source_owner, destination_owner, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        6_150,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_transfer_checked_token_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        TransferCheckedTest::set_up(legacy_token::ID, &source_owner, destination_owner, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        7_800,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_approve_token_program() {
    let source_owner = Keypair::new();
    let delegate = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        ApproveTest::set_up(legacy_token::ID, &source_owner, delegate, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        4_425,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_revoke_token_program() {
    let source_owner = Keypair::new();

    let TestSuccess { tx_meta, .. } = RevokeTest::set_up(legacy_token::ID, &source_owner)
        .await
        .run()
        .await
        .success()
        .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        4_060,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_mint_token_2022_program() {
    let decimals = 9;

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        None, // extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        4_900,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_mint_with_extensions() {
    let decimals = 9;

    // Add each extension separately.

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            close_authority: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_850,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            group_pointer: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            group_member_pointer: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            metadata_pointer: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            non_transferable: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_400,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            permanent_delegate: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_500,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            transfer_fee: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        8_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            transfer_hook: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            confidential_transfer: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_550,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            transfer_fee: true,
            confidential_transfer: true,
            confidential_transfer_fee: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        13_000,
        CU_TOLERANCE
    ));

    // Now add them all.

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
        Some(MintExtensionsForTest {
            close_authority: true,
            group_pointer: true,
            group_member_pointer: true,
            metadata_pointer: true,
            non_transferable: true,
            permanent_delegate: true,
            transfer_fee: true,
            transfer_hook: true,
            confidential_transfer: true,
            confidential_transfer_fee: true,
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        31_150,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_token_account_token_2022_program() {
    let owner = DEFAULT_OWNER;
    let immutable_owner = false;

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        immutable_owner,
        None, // mint_extentions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        5_150,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_ata_token_2022_program() {
    let owner = DEFAULT_OWNER;

    let TestSuccess { tx_meta, .. } = InitAtaTest::set_up(
        spl_token_2022::ID,
        owner,
        false, // idempotent
        None,  // mint_extentions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        17_700,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitAtaTest::set_up(
        spl_token_2022::ID,
        owner,
        true, // idempotent
        None, // mint_extentions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        17_750,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_token_account_with_extensions() {
    let owner = DEFAULT_OWNER;

    // Add each extension separately.

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        false, // immutable_owner
        Some(MintExtensionsForTest {
            transfer_fee: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_450,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        false, // immutable_owner
        Some(MintExtensionsForTest {
            non_transferable: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_600,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        false, // immutable_owner
        Some(MintExtensionsForTest {
            transfer_hook: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_450,
        CU_TOLERANCE
    ));

    // With immutable owner.

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        true, // immutable_owner
        Some(MintExtensionsForTest {
            transfer_fee: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        8_050,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        true, // immutable_owner
        Some(MintExtensionsForTest {
            non_transferable: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_600,
        CU_TOLERANCE
    ));

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        true, // immutable_owner
        Some(MintExtensionsForTest {
            transfer_hook: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        8_050,
        CU_TOLERANCE
    ));

    // Now add them all.

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        true, // immutable_owner
        Some(MintExtensionsForTest {
            transfer_fee: true,
            non_transferable: true,
            transfer_hook: true,
            ..Default::default()
        }),
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        7_750,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_get_account_data_size_token_2022_program() {
    let extensions = [ExtensionType::ImmutableOwner];

    let TestSuccess { tx_meta, .. } =
        GetAccountDataSizeTest::set_up(spl_token_2022::ID, &extensions)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        2_550,
        CU_TOLERANCE
    ));
    assert_eq!(
        tx_meta.return_data,
        Some(TransactionReturnData {
            program_id: ID,
            data: 170_u64.to_le_bytes().to_vec(),
        })
    );
}

#[tokio::test]
async fn test_init_token_account_token_2022_program_immutable_owner() {
    let owner = DEFAULT_OWNER;
    let immutable_owner = true;

    let TestSuccess { tx_meta, .. } = InitTokenAccountTest::set_up(
        spl_token_2022::ID,
        owner,
        immutable_owner,
        None, // mint_extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_750,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_mint_to_token_2022_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = true;

    let TestSuccess { tx_meta, .. } =
        MintToTest::set_up(spl_token_2022::ID, destination_owner, amount, optimized)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        2_550,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_suboptimal_mint_to_token_2022_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = false;

    let TestSuccess { tx_meta, .. } =
        MintToTest::set_up(spl_token_2022::ID, destination_owner, amount, optimized)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,500 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        3_000,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_burn_token_2022_program() {
    let source_owner = Keypair::new();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } = BurnTest::set_up(spl_token_2022::ID, &source_owner, amount)
        .await
        .run()
        .await
        .success()
        .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        2_600,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_transfer_token_2022_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        TransferTest::set_up(spl_token_2022::ID, &source_owner, destination_owner, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        2_750,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_transfer_checked_token_2022_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        TransferCheckedTest::set_up(spl_token_2022::ID, &source_owner, destination_owner, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        4_000,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_approve_token_2022_program() {
    let source_owner = Keypair::new();
    let delegate = Pubkey::new_unique();
    let amount = 420_420;

    let TestSuccess { tx_meta, .. } =
        ApproveTest::set_up(spl_token_2022::ID, &source_owner, delegate, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        2_400,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_revoke_token_2022_program() {
    let source_owner = Keypair::new();

    let TestSuccess { tx_meta, .. } = RevokeTest::set_up(spl_token_2022::ID, &source_owner)
        .await
        .run()
        .await
        .success()
        .unwrap();
    // No PDA addresses found, so no CU adjustments.
    assert!(is_compute_units_within(
        tx_meta.compute_units_consumed,
        2_100,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_mint_token_program_and_freeze_authority() {
    let decimals = 9;
    let freeze_authority = Pubkey::new_unique();

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        legacy_token::ID,
        decimals,
        freeze_authority.into(),
        None, // extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        6_450,
        CU_TOLERANCE
    ));
}

#[tokio::test]
async fn test_init_mint_token_2022_program_and_freeze_authority() {
    let decimals = 9;
    let freeze_authority = Pubkey::new_unique();

    let TestSuccess { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        freeze_authority.into(),
        None, // extensions
    )
    .await
    .run()
    .await
    .success()
    .unwrap();
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,500
    // CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_500;
    assert!(is_compute_units_within(
        adjusted_compute_units_consumed,
        5_050,
        CU_TOLERANCE
    ));
}

struct InitMintTest {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    decimals: u8,
    freeze_authority: Option<Pubkey>,
    extensions: MintExtensionsForTest,
}

#[derive(Default, Clone, Copy)]
struct MintExtensionsForTest {
    close_authority: bool,
    group_pointer: bool,
    group_member_pointer: bool,
    metadata_pointer: bool,
    non_transferable: bool,
    permanent_delegate: bool,
    transfer_fee: bool,
    transfer_hook: bool,
    confidential_transfer: bool,
    confidential_transfer_fee: bool,
}

impl InitMintTest {
    async fn set_up(
        token_program_id: Pubkey,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
        extensions: Option<MintExtensionsForTest>,
    ) -> Self {
        let (banks_client, payer, recent_blockhash) =
            ProgramTest::new("example_token_management", ID, None)
                .start()
                .await;

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            decimals,
            freeze_authority,
            extensions: extensions.unwrap_or_default(),
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            decimals,
            freeze_authority,
            extensions:
                MintExtensionsForTest {
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
                },
        } = self;
        dbg!(
            token_program_id,
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
        );

        let (new_mint_addr, mint_bump) = state::find_mint_address();
        assert_eq!(mint_bump, 252);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new(new_mint_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::InitMint(InitMintWithExtensionsData {
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
            }))
            .unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check that mint exists.
        let mint_account = banks_client
            .get_account(new_mint_addr)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(mint_account.owner, token_program_id);

        let mint_data = StateWithExtensionsOwned::<Mint>::unpack(mint_account.data).unwrap();
        let (mint_authority_addr, mint_authority_bump) = state::find_authority_address();
        assert_eq!(mint_authority_bump, 255);
        assert_eq!(
            mint_data.base,
            Mint {
                mint_authority: mint_authority_addr.into(),
                freeze_authority: freeze_authority.into(),
                decimals,
                is_initialized: true,
                supply: 0,
            }
        );

        let close_authority_result = mint_data
                .get_extension::<spl_token_2022::extension::mint_close_authority::MintCloseAuthority>();
        if close_authority {
            let close_authority_data = close_authority_result.unwrap();
            assert_eq!(close_authority_data.close_authority.0, mint_authority_addr);
        } else {
            assert!(close_authority_result.is_err());
        }

        let group_pointer_result =
            mint_data.get_extension::<spl_token_2022::extension::group_pointer::GroupPointer>();
        if group_pointer {
            let group_pointer_data = group_pointer_result.unwrap();
            assert_eq!(group_pointer_data.authority.0, mint_authority_addr);
            assert_eq!(group_pointer_data.group_address.0, new_mint_addr);
        } else {
            assert!(group_pointer_result.is_err());
        }

        let group_member_pointer_result = mint_data
            .get_extension::<spl_token_2022::extension::group_member_pointer::GroupMemberPointer>();
        if group_member_pointer {
            let group_member_pointer_data = group_member_pointer_result.unwrap();
            assert_eq!(group_member_pointer_data.authority.0, mint_authority_addr);
            assert_eq!(group_member_pointer_data.member_address.0, new_mint_addr);
        } else {
            assert!(group_member_pointer_result.is_err());
        }

        let metadata_pointer_result = mint_data
            .get_extension::<spl_token_2022::extension::metadata_pointer::MetadataPointer>(
        );
        if metadata_pointer {
            let metadata_pointer_data = metadata_pointer_result.unwrap();
            assert_eq!(metadata_pointer_data.authority.0, mint_authority_addr);
            assert_eq!(metadata_pointer_data.metadata_address.0, new_mint_addr);
        } else {
            assert!(metadata_pointer_result.is_err());
        }

        let non_transferable_result = mint_data
            .get_extension::<spl_token_2022::extension::non_transferable::NonTransferable>(
        );
        if non_transferable {
            assert!(non_transferable_result.is_ok());
        } else {
            assert!(non_transferable_result.is_err());
        }

        let permanent_delegate_result =
            mint_data
                .get_extension::<spl_token_2022::extension::permanent_delegate::PermanentDelegate>(
                );
        if permanent_delegate {
            let permanent_delegate_data = permanent_delegate_result.unwrap();
            assert_eq!(permanent_delegate_data.delegate.0, mint_authority_addr);
        } else {
            assert!(permanent_delegate_result.is_err());
        }

        let transfer_fee_config_result =
            mint_data.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>();
        if transfer_fee {
            let transfer_fee_config_data = transfer_fee_config_result.unwrap();
            assert_eq!(
                transfer_fee_config_data.transfer_fee_config_authority.0,
                mint_authority_addr
            );
            assert_eq!(
                transfer_fee_config_data.withdraw_withheld_authority.0,
                mint_authority_addr
            );
        } else {
            assert!(transfer_fee_config_result.is_err());
        }

        let transfer_hook_result =
            mint_data.get_extension::<spl_token_2022::extension::transfer_hook::TransferHook>();
        if transfer_hook {
            let transfer_hook_data = transfer_hook_result.unwrap();
            assert_eq!(transfer_hook_data.authority.0, mint_authority_addr);
            assert_eq!(transfer_hook_data.program_id.0, ID);
        } else {
            assert!(transfer_hook_result.is_err());
        }

        let confidential_transfer_result = mint_data
            .get_extension::<spl_token_2022::extension::confidential_transfer::ConfidentialTransferMint>(
        );
        if confidential_transfer {
            let confidential_transfer_data = confidential_transfer_result.unwrap();
            assert_eq!(confidential_transfer_data.authority.0, mint_authority_addr);
            assert!(bool::from(
                confidential_transfer_data.auto_approve_new_accounts
            ));
            assert!(confidential_transfer_data
                .auditor_elgamal_pubkey
                .equals(&[1; 32].into()));
        } else {
            assert!(confidential_transfer_result.is_err());
        }

        let confidential_transfer_fee_config_result = mint_data
            .get_extension::<spl_token_2022::extension::confidential_transfer_fee::ConfidentialTransferFeeConfig>(
        );
        if confidential_transfer_fee {
            let confidential_transfer_fee_config_data =
                confidential_transfer_fee_config_result.unwrap();
            assert_eq!(
                confidential_transfer_fee_config_data.authority.0,
                mint_authority_addr
            );
            assert_eq!(
                confidential_transfer_fee_config_data
                    .withdraw_withheld_authority_elgamal_pubkey
                    .to_string(),
                String::from("AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=")
            );
        } else {
            assert!(confidential_transfer_fee_config_result.is_err());
        }

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct InitTokenAccountTest {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    owner: Pubkey,
    immutable_owner: bool,
    mint_extensions: MintExtensionsForTest,
}

impl InitTokenAccountTest {
    async fn set_up(
        token_program_id: Pubkey,
        owner: Pubkey,
        immutable_owner: bool,
        mint_extensions: Option<MintExtensionsForTest>,
    ) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitMintTest::set_up(
            token_program_id,
            9,    // decimals
            None, // freeze_authority
            mint_extensions,
        )
        .await
        .run()
        .await
        .success()
        .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            immutable_owner,
            mint_extensions: mint_extensions.unwrap_or_default(),
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            immutable_owner,
            mint_extensions:
                MintExtensionsForTest {
                    transfer_fee,
                    non_transferable,
                    transfer_hook,
                    ..
                },
        } = self;
        dbg!(
            token_program_id,
            owner,
            immutable_owner,
            transfer_fee,
            non_transferable,
            transfer_hook
        );

        let (mint_addr, mint_bump) = state::find_mint_address();
        assert_eq!(mint_bump, 252);

        let (new_token_account_addr, token_account_bump) =
            state::find_token_account_address(&owner);
        if owner == DEFAULT_OWNER {
            assert_eq!(token_account_bump, 252);
        }

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(mint_addr, false),
                AccountMeta::new(new_token_account_addr, false),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::InitTokenAccount {
                owner,
                immutable_owner,
            })
            .unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check that token account exists.
        let token_account = banks_client
            .get_account(new_token_account_addr)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(token_account.owner, token_program_id);

        let token_account_data =
            StateWithExtensionsOwned::<Account>::unpack(token_account.data).unwrap();
        assert_eq!(
            token_account_data.base,
            Account {
                mint: mint_addr,
                owner,
                amount: 0,
                delegate: Default::default(),
                state: spl_token_2022::state::AccountState::Initialized,
                is_native: Default::default(),
                delegated_amount: 0,
                close_authority: Default::default(),
            }
        );

        let transfer_fee_amount_result = token_account_data
            .get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeAmount>(
        );
        if transfer_fee {
            assert!(transfer_fee_amount_result.is_ok());
        } else {
            assert!(transfer_fee_amount_result.is_err());
        };

        let non_transferable_account = token_account_data
            .get_extension::<spl_token_2022::extension::non_transferable::NonTransferableAccount>(
        );
        let immutable_owner_result = token_account_data
            .get_extension::<spl_token_2022::extension::immutable_owner::ImmutableOwner>(
        );
        if non_transferable {
            assert!(non_transferable_account.is_ok());
            assert!(immutable_owner_result.is_ok());
        } else if immutable_owner {
            assert!(non_transferable_account.is_err());
            assert!(immutable_owner_result.is_ok());
        } else {
            assert!(non_transferable_account.is_err());
            assert!(immutable_owner_result.is_err());
        }

        let transfer_hook_account = token_account_data
            .get_extension::<spl_token_2022::extension::transfer_hook::TransferHookAccount>(
        );
        if transfer_hook {
            assert!(transfer_hook_account.is_ok());
        } else {
            assert!(transfer_hook_account.is_err());
        }

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct InitAtaTest {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    owner: Pubkey,
    idempotent: bool,
    mint_extensions: MintExtensionsForTest,
}

impl InitAtaTest {
    async fn set_up(
        token_program_id: Pubkey,
        owner: Pubkey,
        idempotent: bool,
        mint_extensions: Option<MintExtensionsForTest>,
    ) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitMintTest::set_up(
            token_program_id,
            9,    // decimals
            None, // freeze_authority
            mint_extensions,
        )
        .await
        .run()
        .await
        .success()
        .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            idempotent,
            mint_extensions: mint_extensions.unwrap_or_default(),
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            idempotent,
            mint_extensions:
                MintExtensionsForTest {
                    transfer_fee,
                    non_transferable,
                    transfer_hook,
                    ..
                },
        } = self;
        dbg!(
            token_program_id,
            owner,
            idempotent,
            transfer_fee,
            non_transferable,
            transfer_hook
        );

        let (mint_addr, mint_bump) = state::find_mint_address();
        assert_eq!(mint_bump, 252);

        let (new_ata_addr, ata_bump) = AssociatedTokenAccountSeeds {
            owner: &owner,
            token_program_id: &token_program_id,
            mint: &mint_addr,
        }
        .find_program_address(None);
        if token_program_id == legacy_token::ID {
            assert_eq!(ata_bump, 254);
        } else {
            assert_eq!(ata_bump, 255);
        }

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(new_ata_addr, false),
                AccountMeta::new_readonly(owner, false),
                AccountMeta::new_readonly(mint_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(sealevel_tools::account::ata::ID, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::InitAta(idempotent)).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check that token account exists.
        let token_account = banks_client
            .get_account(new_ata_addr)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(token_account.owner, token_program_id);

        let token_account_data =
            StateWithExtensionsOwned::<Account>::unpack(token_account.data).unwrap();
        assert_eq!(
            token_account_data.base,
            Account {
                mint: mint_addr,
                owner,
                amount: 0,
                delegate: Default::default(),
                state: spl_token_2022::state::AccountState::Initialized,
                is_native: Default::default(),
                delegated_amount: 0,
                close_authority: Default::default(),
            }
        );

        let transfer_fee_amount_result = token_account_data
            .get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeAmount>(
        );
        if transfer_fee {
            assert!(transfer_fee_amount_result.is_ok());
        } else {
            assert!(transfer_fee_amount_result.is_err());
        };

        let non_transferable_account = token_account_data
            .get_extension::<spl_token_2022::extension::non_transferable::NonTransferableAccount>(
        );
        let immutable_owner_result = token_account_data
            .get_extension::<spl_token_2022::extension::immutable_owner::ImmutableOwner>(
        );
        if non_transferable {
            assert!(non_transferable_account.is_ok());
            assert!(immutable_owner_result.is_ok());
        } else if token_program_id == token_extensions::ID {
            assert!(non_transferable_account.is_err());
            assert!(immutable_owner_result.is_ok());
        } else {
            assert!(non_transferable_account.is_err());
            assert!(immutable_owner_result.is_err());
        }

        let transfer_hook_account = token_account_data
            .get_extension::<spl_token_2022::extension::transfer_hook::TransferHookAccount>(
        );
        if transfer_hook {
            assert!(transfer_hook_account.is_ok());
        } else {
            assert!(transfer_hook_account.is_err());
        }

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct MintToTest {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    destination_owner: Pubkey,
    amount: u64,
    optimized: bool,
}

impl MintToTest {
    async fn set_up(
        token_program_id: Pubkey,
        destination_owner: Pubkey,
        amount: u64,
        optimized: bool,
    ) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitTokenAccountTest::set_up(token_program_id, destination_owner, false, None)
            .await
            .run()
            .await
            .success()
            .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            destination_owner,
            amount,
            optimized,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            destination_owner,
            amount,
            optimized,
        } = self;

        let (mint_addr, _) = state::find_mint_address();

        // Check the mint supply.
        let mint_supply = StateWithExtensionsOwned::<Mint>::unpack(
            banks_client
                .get_account(mint_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .supply;
        assert_eq!(mint_supply, 0);

        let (destination_token_account_addr, _) =
            state::find_token_account_address(&destination_owner);

        // Check the token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        let (authority_addr, authority_bump) = state::find_authority_address();
        assert_eq!(authority_bump, 255);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(mint_addr, false),
                AccountMeta::new(destination_token_account_addr, false),
                AccountMeta::new_readonly(authority_addr, false),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&if optimized {
                ProgramInstruction::MintTo(amount)
            } else {
                ProgramInstruction::SuboptimalMintTo(amount)
            })
            .unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        // Check the mint supply.
        let mint_supply = StateWithExtensionsOwned::<Mint>::unpack(
            banks_client
                .get_account(mint_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .supply;
        assert_eq!(mint_supply, amount);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct BurnTest<'a> {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    source_owner: &'a Keypair,
    amount: u64,
}

impl<'a> BurnTest<'a> {
    async fn set_up(token_program_id: Pubkey, source_owner: &'a Keypair, amount: u64) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = MintToTest::set_up(token_program_id, source_owner.pubkey(), amount, true)
            .await
            .run()
            .await
            .success()
            .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            amount,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            amount,
        } = self;

        let (mint_addr, _) = state::find_mint_address();

        // Check the mint supply.
        let mint_supply = StateWithExtensionsOwned::<Mint>::unpack(
            banks_client
                .get_account(mint_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .supply;
        assert_eq!(mint_supply, amount);

        let (source_token_account_addr, _) =
            state::find_token_account_address(&source_owner.pubkey());

        // Check the token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(source_token_account_addr, false),
                AccountMeta::new(mint_addr, false),
                AccountMeta::new_readonly(source_owner.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::Burn(amount)).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        // Check the mint supply.
        let mint_supply = StateWithExtensionsOwned::<Mint>::unpack(
            banks_client
                .get_account(mint_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .supply;
        assert_eq!(mint_supply, 0);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct TransferTest<'a> {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    source_owner: &'a Keypair,
    destination_owner: Pubkey,
    amount: u64,
}

impl<'a> TransferTest<'a> {
    async fn set_up(
        token_program_id: Pubkey,
        source_owner: &'a Keypair,
        destination_owner: Pubkey,
        amount: u64,
    ) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = MintToTest::set_up(token_program_id, source_owner.pubkey(), amount, true)
            .await
            .run()
            .await
            .success()
            .unwrap();

        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitTokenAccountTest {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner: destination_owner,
            immutable_owner: false,
            mint_extensions: Default::default(),
        }
        .run()
        .await
        .success()
        .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            destination_owner,
            amount,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            destination_owner,
            amount,
        } = self;

        let (source_token_account_addr, _) =
            state::find_token_account_address(&source_owner.pubkey());
        let (destination_token_account_addr, _) =
            state::find_token_account_address(&destination_owner);

        // Check the source token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        // Check the destination token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(source_token_account_addr, false),
                AccountMeta::new(destination_token_account_addr, false),
                AccountMeta::new_readonly(source_owner.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::Transfer(amount)).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the source token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        // Check the destination token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct TransferCheckedTest<'a>(TransferTest<'a>);

impl<'a> TransferCheckedTest<'a> {
    async fn set_up(
        token_program_id: Pubkey,
        source_owner: &'a Keypair,
        destination_owner: Pubkey,
        amount: u64,
    ) -> Self {
        Self(TransferTest::set_up(token_program_id, source_owner, destination_owner, amount).await)
    }

    async fn run(self) -> TestResult {
        let TransferTest {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            destination_owner,
            amount,
        } = self.0;

        let (mint_addr, _) = state::find_mint_address();
        let (source_token_account_addr, _) =
            state::find_token_account_address(&source_owner.pubkey());
        let (destination_token_account_addr, _) =
            state::find_token_account_address(&destination_owner);

        // Fetch decimals.
        let decimals = StateWithExtensionsOwned::<Mint>::unpack(
            banks_client
                .get_account(mint_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .decimals;

        // Check the source token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        // Check the destination token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(source_token_account_addr, false),
                AccountMeta::new_readonly(mint_addr, false),
                AccountMeta::new(destination_token_account_addr, false),
                AccountMeta::new_readonly(source_owner.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::TransferChecked { amount, decimals }).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the source token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, 0);

        // Check the destination token account amount.
        let token_account_balance = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(destination_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base
        .amount;
        assert_eq!(token_account_balance, amount);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct ApproveTest<'a> {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    source_owner: &'a Keypair,
    delegated_authority: Pubkey,
    amount: u64,
}

impl<'a> ApproveTest<'a> {
    async fn set_up(
        token_program_id: Pubkey,
        source_owner: &'a Keypair,
        delegated_authority: Pubkey,
        amount: u64,
    ) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitTokenAccountTest::set_up(token_program_id, source_owner.pubkey(), false, None)
            .await
            .run()
            .await
            .success()
            .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            delegated_authority,
            amount,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            delegated_authority,
            amount,
        } = self;

        let (source_token_account_addr, _) =
            state::find_token_account_address(&source_owner.pubkey());

        // Check the token account delegate.
        let Account {
            delegate,
            delegated_amount,
            ..
        } = StateWithExtensionsOwned::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base;
        assert!(delegate.is_none());
        assert_eq!(delegated_amount, 0);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(source_token_account_addr, false),
                AccountMeta::new_readonly(delegated_authority, false),
                AccountMeta::new_readonly(source_owner.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::Approve(amount)).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the token account delegate.
        let Account {
            delegate,
            delegated_amount,
            ..
        } = StateWithExtensionsOwned::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base;
        assert_eq!(delegate, COption::Some(delegated_authority));
        assert_eq!(delegated_amount, amount);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct RevokeTest<'a> {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    source_owner: &'a Keypair,
}

impl<'a> RevokeTest<'a> {
    async fn set_up(token_program_id: Pubkey, source_owner: &'a Keypair) -> Self {
        let delegated_authority = Pubkey::new_unique();
        let amount = 420_420;

        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = ApproveTest::set_up(token_program_id, source_owner, delegated_authority, amount)
            .await
            .run()
            .await
            .success()
            .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
        } = self;

        let (source_token_account_addr, _) =
            state::find_token_account_address(&source_owner.pubkey());

        // Check the token account delegate.
        let Account {
            delegate,
            delegated_amount,
            ..
        } = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base;
        assert!(delegate.is_some());
        assert_ne!(delegated_amount, 0);

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new(source_token_account_addr, false),
                AccountMeta::new_readonly(source_owner.pubkey(), true),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::Revoke).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        // Check the token account delegate.
        let Account {
            delegate,
            delegated_amount,
            ..
        } = StateWithExtensionsOwned::<Account>::unpack(
            banks_client
                .get_account(source_token_account_addr)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
        .base;
        assert!(delegate.is_none());
        assert_eq!(delegated_amount, 0);

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}

struct GetAccountDataSizeTest<'a> {
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    token_program_id: Pubkey,
    extensions: &'a [ExtensionType],
}

impl<'a> GetAccountDataSizeTest<'a> {
    async fn set_up(token_program_id: Pubkey, extensions: &'a [ExtensionType]) -> Self {
        let TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitMintTest::set_up(
            token_program_id,
            9,    // decimals
            None, // freeze_authority
            None, // extensions
        )
        .await
        .run()
        .await
        .success()
        .unwrap();

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            extensions,
        }
    }

    async fn run(self) -> TestResult {
        let Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            extensions,
        } = self;

        let (mint_addr, _) = state::find_mint_address();

        let instruction = Instruction {
            program_id: ID,
            accounts: vec![
                AccountMeta::new_readonly(mint_addr, false),
                AccountMeta::new_readonly(token_program_id, false),
            ],
            data: borsh::to_vec(&ProgramInstruction::GetAccountDataSize(ExtensionTypes(
                extensions.to_vec(),
            )))
            .unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();

        if is_program_failure(&ID, &tx_meta.log_messages) {
            return TestResult::Fail(tx_meta);
        }

        TestSuccess {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
        .into()
    }
}
