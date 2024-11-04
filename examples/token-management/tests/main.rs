use example_token_management::{
    instruction::{ExtensionTypes, ProgramInstruction},
    state, ID,
};
use examples_common::program_failed;
use solana_banks_interface::TransactionMetadata;
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
    extension::{
        immutable_owner::ImmutableOwner, BaseStateWithExtensions, ExtensionType,
        StateWithExtensionsOwned,
    },
    state::{Account, Mint},
};

const DEFAULT_OWNER: Pubkey = solana_sdk::pubkey!("Defau1towner1111111111111111111111111111111");

#[tokio::test]
async fn test_init_mint_token_program() {
    let decimals = 9;

    let TestResult { tx_meta, .. } = InitMintTest::set_up(
        spl_token::ID,
        decimals,
        None, // freeze_authority
    )
    .await
    .into_success()
    .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,200
    // CU. The total adjustment is 6,000 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 7_856);
}

#[tokio::test]
async fn test_init_token_account_token_program() {
    let owner = DEFAULT_OWNER;
    let immutable = false;

    let TestResult { tx_meta, .. } = InitTokenAccountTest::set_up(spl_token::ID, owner, immutable)
        .await
        .into_success()
        .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,200 CU. The total adjustment is 9,600 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 10_355);
}

#[tokio::test]
async fn test_get_account_data_size_token_program() {
    let extensions = [ExtensionType::ImmutableOwner];

    let TestResult { tx_meta, .. } = GetAccountDataSizeTest::set_up(spl_token::ID, &extensions)
        .await
        .into_success()
        .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 3_217);
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

    let TestResult { tx_meta, .. } =
        MintToTest::set_up(spl_token::ID, destination_owner, amount, optimized)
            .await
            .into_success()
            .await;
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,200 CU. The total adjustment is 1,200 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_200;
    assert_eq!(adjusted_compute_units_consumed, 6_361);
}

#[tokio::test]
async fn test_suboptimal_mint_to_token_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = false;

    let TestResult { tx_meta, .. } =
        MintToTest::set_up(spl_token::ID, destination_owner, amount, optimized)
            .await
            .into_success()
            .await;
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,200 CU. The total adjustment is 1,200 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_200;
    assert_eq!(adjusted_compute_units_consumed, 6_783);
}

#[tokio::test]
async fn test_burn_token_program() {
    let source_owner = Keypair::new();
    let amount = 420_420;

    let TestResult { tx_meta, .. } = BurnTest::set_up(spl_token::ID, &source_owner, amount)
        .await
        .into_success()
        .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 6_202);
}

#[tokio::test]
async fn test_transfer_token_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        TransferTest::set_up(spl_token::ID, &source_owner, destination_owner, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 6_121);
}

#[tokio::test]
async fn test_transfer_checked_token_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        TransferCheckedTest::set_up(spl_token::ID, &source_owner, destination_owner, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 7_787);
}

#[tokio::test]
async fn test_approve_token_program() {
    let source_owner = Keypair::new();
    let delegate = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        ApproveTest::set_up(spl_token::ID, &source_owner, delegate, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 4_401);
}

#[tokio::test]
async fn test_revoke_token_program() {
    let source_owner = Keypair::new();

    let TestResult { tx_meta, .. } = RevokeTest::set_up(spl_token::ID, &source_owner)
        .await
        .into_success()
        .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 4_039);
}

#[tokio::test]
async fn test_init_mint_token_2022_program() {
    let decimals = 9;

    let TestResult { tx_meta, .. } = InitMintTest::set_up(
        spl_token_2022::ID,
        decimals,
        None, // freeze_authority
    )
    .await
    .into_success()
    .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,200
    // CU. The total adjustment is 6,000 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 8_224);
}

#[tokio::test]
async fn test_init_token_account_token_2022_program() {
    let owner = DEFAULT_OWNER;
    let immutable = false;

    let TestResult { tx_meta, .. } =
        InitTokenAccountTest::set_up(spl_token_2022::ID, owner, immutable)
            .await
            .into_success()
            .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,200 CU. The total adjustment is 9,600 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 10_498);
}

#[tokio::test]
async fn test_get_account_data_size_token_2022_program() {
    let extensions = [ExtensionType::ImmutableOwner];

    let TestResult { tx_meta, .. } =
        GetAccountDataSizeTest::set_up(spl_token_2022::ID, &extensions)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 4_056);
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
    let immutable = true;

    let TestResult { tx_meta, .. } =
        InitTokenAccountTest::set_up(spl_token_2022::ID, owner, immutable)
            .await
            .into_success()
            .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Token account bump
    // is 252, which requires 4 iterations to find the token account key. Each bump iteration costs
    // 1,200 CU. The total adjustment is 9,600 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 8 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 13_538);
}

#[tokio::test]
async fn test_mint_to_token_2022_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = true;

    let TestResult { tx_meta, .. } =
        MintToTest::set_up(spl_token_2022::ID, destination_owner, amount, optimized)
            .await
            .into_success()
            .await;
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,200 CU. The total adjustment is 1,200 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_200;
    assert_eq!(adjusted_compute_units_consumed, 6_900);
}

#[tokio::test]
async fn test_suboptimal_mint_to_token_2022_program() {
    let destination_owner = DEFAULT_OWNER;
    let amount = 420_420;
    let optimized = false;

    let TestResult { tx_meta, .. } =
        MintToTest::set_up(spl_token_2022::ID, destination_owner, amount, optimized)
            .await
            .into_success()
            .await;
    // NOTE: Mint authority bump is 255, which requires 1 iteration to find the mint authority key.
    // Each bump iteration costs 1,200 CU. The total adjustment is 1,200 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 1_200;
    assert_eq!(adjusted_compute_units_consumed, 7_316);
}

#[tokio::test]
async fn test_burn_token_2022_program() {
    let source_owner = Keypair::new();
    let amount = 420_420;

    let TestResult { tx_meta, .. } = BurnTest::set_up(spl_token_2022::ID, &source_owner, amount)
        .await
        .into_success()
        .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 6_826);
}

#[tokio::test]
async fn test_transfer_token_2022_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        TransferTest::set_up(spl_token_2022::ID, &source_owner, destination_owner, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 7_597);
}

#[tokio::test]
async fn test_transfer_checked_token_2022_program() {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        TransferCheckedTest::set_up(spl_token_2022::ID, &source_owner, destination_owner, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 9_984);
}

#[tokio::test]
async fn test_approve_token_2022_program() {
    let source_owner = Keypair::new();
    let delegate = Pubkey::new_unique();
    let amount = 420_420;

    let TestResult { tx_meta, .. } =
        ApproveTest::set_up(spl_token_2022::ID, &source_owner, delegate, amount)
            .await
            .into_success()
            .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 4_902);
}

#[tokio::test]
async fn test_revoke_token_2022_program() {
    let source_owner = Keypair::new();

    let TestResult { tx_meta, .. } = RevokeTest::set_up(spl_token_2022::ID, &source_owner)
        .await
        .into_success()
        .await;
    // No PDA addresses found, so no CU adjustments.
    assert_eq!(tx_meta.compute_units_consumed, 4_527);
}

#[tokio::test]
async fn test_init_mint_token_program_and_freeze_authority() {
    let decimals = 9;
    let freeze_authority = Pubkey::new_unique();

    let TestResult { tx_meta, .. } =
        InitMintTest::set_up(spl_token::ID, decimals, freeze_authority.into())
            .await
            .into_success()
            .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,200
    // CU. The total adjustment is 6,000 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 8_231);
}

#[tokio::test]
async fn test_init_mint_token_2022_program_and_freeze_authority() {
    let decimals = 9;
    let freeze_authority = Pubkey::new_unique();

    let TestResult { tx_meta, .. } =
        InitMintTest::set_up(spl_token_2022::ID, decimals, freeze_authority.into())
            .await
            .into_success()
            .await;
    // NOTE: Mint bump is 252, which requires 4 iterations to find the mint key. Authority bump is
    // 255, which requires 1 iteration to find the authority key. Each bump iteration costs 1,200
    // CU. The total adjustment is 6,000 CU.
    let adjusted_compute_units_consumed = tx_meta.compute_units_consumed - 5 * 1_200;
    assert_eq!(adjusted_compute_units_consumed, 8_585);
}

pub struct TestResult {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub tx_meta: TransactionMetadata,
}

pub struct InitMintTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub decimals: u8,
    pub freeze_authority: Option<Pubkey>,
}

impl InitMintTest {
    async fn set_up(
        token_program_id: Pubkey,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
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
        }
    }

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            decimals,
            freeze_authority,
        } = self;

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
            data: borsh::to_vec(&ProgramInstruction::InitMint {
                decimals,
                freeze_authority,
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
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct InitTokenAccountTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub owner: Pubkey,
    pub immutable: bool,
}

impl InitTokenAccountTest {
    async fn set_up(token_program_id: Pubkey, owner: Pubkey, immutable: bool) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitMintTest::set_up(
            token_program_id,
            9,    // decimals
            None, // freeze_authority
        )
        .await
        .into_success()
        .await;

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            immutable,
        }
    }

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            owner,
            immutable,
        } = self;

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
            data: borsh::to_vec(&ProgramInstruction::InitTokenAccount { owner, immutable })
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
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        let immutable_owner_result = token_account_data.get_extension::<ImmutableOwner>();
        assert!(if immutable {
            immutable_owner_result.is_ok()
        } else {
            immutable_owner_result.is_err()
        });

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct MintToTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub destination_owner: Pubkey,
    pub amount: u64,
    pub optimized: bool,
}

impl MintToTest {
    async fn set_up(
        token_program_id: Pubkey,
        destination_owner: Pubkey,
        amount: u64,
        optimized: bool,
    ) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitTokenAccountTest::set_up(token_program_id, destination_owner, false)
            .await
            .into_success()
            .await;

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

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
                ProgramInstruction::MintTo { amount }
            } else {
                ProgramInstruction::SuboptimalMintTo { amount }
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
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct BurnTest<'a> {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub source_owner: &'a Keypair,
    pub amount: u64,
}

impl<'a> BurnTest<'a> {
    async fn set_up(token_program_id: Pubkey, source_owner: &'a Keypair, amount: u64) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = MintToTest::set_up(token_program_id, source_owner.pubkey(), amount, true)
            .await
            .into_success()
            .await;

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
            amount,
        }
    }

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
            data: borsh::to_vec(&ProgramInstruction::Burn { amount }).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct TransferTest<'a> {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub source_owner: &'a Keypair,
    pub destination_owner: Pubkey,
    pub amount: u64,
}

impl<'a> TransferTest<'a> {
    async fn set_up(
        token_program_id: Pubkey,
        source_owner: &'a Keypair,
        destination_owner: Pubkey,
        amount: u64,
    ) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = MintToTest::set_up(token_program_id, source_owner.pubkey(), amount, true)
            .await
            .into_success()
            .await;

        let TestResult {
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
            immutable: false,
        }
        .into_success()
        .await;

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

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
            data: borsh::to_vec(&ProgramInstruction::Transfer { amount }).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
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

    async fn into_success(self) -> TestResult {
        let TransferTest {
            mut banks_client,
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
        transaction.sign(&[&payer, &source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct ApproveTest<'a> {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub source_owner: &'a Keypair,
    pub delegated_authority: Pubkey,
    pub amount: u64,
}

impl<'a> ApproveTest<'a> {
    async fn set_up(
        token_program_id: Pubkey,
        source_owner: &'a Keypair,
        delegated_authority: Pubkey,
        amount: u64,
    ) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitTokenAccountTest::set_up(token_program_id, source_owner.pubkey(), false)
            .await
            .into_success()
            .await;

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

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
            data: borsh::to_vec(&ProgramInstruction::Approve { amount }).unwrap(),
        };
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct RevokeTest<'a> {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub source_owner: &'a Keypair,
}

impl<'a> RevokeTest<'a> {
    async fn set_up(token_program_id: Pubkey, source_owner: &'a Keypair) -> Self {
        let delegated_authority = Pubkey::new_unique();
        let amount = 420_420;

        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = ApproveTest::set_up(token_program_id, source_owner, delegated_authority, amount)
            .await
            .into_success()
            .await;

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            source_owner,
        }
    }

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
        transaction.sign(&[&payer, &source_owner], recent_blockhash);

        let tx_meta = banks_client
            .process_transaction_with_metadata(transaction)
            .await
            .unwrap()
            .metadata
            .unwrap();
        assert!(!program_failed(&ID, &tx_meta.log_messages));

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

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}

pub struct GetAccountDataSizeTest<'a> {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub token_program_id: Pubkey,
    pub extensions: &'a [ExtensionType],
}

impl<'a> GetAccountDataSizeTest<'a> {
    async fn set_up(token_program_id: Pubkey, extensions: &'a [ExtensionType]) -> Self {
        let TestResult {
            banks_client,
            payer,
            recent_blockhash,
            ..
        } = InitMintTest::set_up(
            token_program_id,
            9,    // decimals
            None, // freeze_authority
        )
        .await
        .into_success()
        .await;

        Self {
            banks_client,
            payer,
            recent_blockhash,
            token_program_id,
            extensions,
        }
    }

    async fn into_success(self) -> TestResult {
        let Self {
            mut banks_client,
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
        assert!(!program_failed(&ID, &tx_meta.log_messages));

        TestResult {
            banks_client,
            payer,
            recent_blockhash,
            tx_meta,
        }
    }
}
