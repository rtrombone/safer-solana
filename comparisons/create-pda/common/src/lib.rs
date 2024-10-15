use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use create_pda_using_anchor::Thing;
use solana_banks_interface::TransactionMetadata;
use solana_program_test::BanksClient;
use solana_sdk::{
    hash::Hash, instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, system_program, transaction::Transaction,
};

pub struct InitThingTest {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub program_id: Pubkey,
}

pub async fn init_thing_for_test(
    InitThingTest {
        mut banks_client,
        payer,
        recent_blockhash,
        program_id,
    }: InitThingTest,
) -> TransactionMetadata {
    let (new_thing_addr, _) = Pubkey::find_program_address(&[b"thing"], &program_id);

    let instruction = Instruction {
        program_id,
        accounts: create_pda_using_anchor::accounts::InitThing {
            payer: payer.pubkey(),
            new_thing: new_thing_addr,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: create_pda_using_anchor::instruction::InitThing {}.data(),
    };
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();

    // Check the new_thing account.
    let account_data = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .data;
    let thing_data = Thing::try_deserialize(&mut &account_data[..]).unwrap();
    assert_eq!(thing_data, Thing { data: 69 });

    tx_meta
}

pub async fn init_thing_already_having_lamports_for_test(
    InitThingTest {
        mut banks_client,
        payer,
        recent_blockhash,
        program_id,
    }: InitThingTest,
) -> TransactionMetadata {
    let transfer_instruction = system_instruction::transfer(
        &payer.pubkey(),
        &Pubkey::find_program_address(&[b"thing"], &program_id).0,
        1,
    );

    let (new_thing_addr, _) = Pubkey::find_program_address(&[b"thing"], &program_id);

    let init_thing_instruction = Instruction {
        program_id,
        accounts: create_pda_using_anchor::accounts::InitThing {
            payer: payer.pubkey(),
            new_thing: new_thing_addr,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: create_pda_using_anchor::instruction::InitThing {}.data(),
    };
    let mut transaction = Transaction::new_with_payer(
        &[transfer_instruction, init_thing_instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();

    // Check the new_thing account.
    let account_data = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .data;
    let thing_data = Thing::try_deserialize(&mut &account_data[..]).unwrap();
    assert_eq!(thing_data, Thing { data: 69 });

    tx_meta
}
