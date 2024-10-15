use anchor_lang::{InstructionData, ToAccountMetas};
use solana_banks_interface::TransactionMetadata;
use solana_program_test::BanksClient;
use solana_sdk::{
    hash::Hash, instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program, transaction::Transaction,
};

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const TOKEN_2022_PROGRAM_ID: Pubkey = spl_token_2022::ID;

pub struct InitMintTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub program_id: Pubkey,
    pub token_program_id: Pubkey,
    pub decimals: u8,
    pub mint_authority: Pubkey,
    pub freeze_authority: Option<Pubkey>,
}

pub async fn init_mint_for_test(
    InitMintTest {
        mut banks_client,
        payer,
        recent_blockhash,
        program_id,
        token_program_id,
        decimals,
        mint_authority,
        freeze_authority,
    }: InitMintTest,
) -> TransactionMetadata {
    let (new_mint_addr, _) = Pubkey::find_program_address(&[b"mint"], &program_id);

    let instruction = Instruction {
        program_id,
        accounts: create_mint_pda_using_anchor::accounts::InitMint {
            payer: payer.pubkey(),
            new_mint: new_mint_addr,
            token_program: token_program_id,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: create_mint_pda_using_anchor::instruction::InitMint {
            decimals,
            mint_authority,
            freeze_authority,
        }
        .data(),
    };
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap()
}
