use example_account_management::{instruction::ProgramInstruction, state::Thing};
use sealevel_tools::account::BorshAccountSerde;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    instruction::AccountMeta, signer::Signer, system_instruction, system_program,
    transaction::Transaction,
};

#[tokio::test]
async fn test_thing() {
    // Init.
    let value = 69;

    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "example_account_management",
        example_account_management::ID,
        processor!(example_account_management::entrypoint::process_instruction),
    )
    .start()
    .await;

    let (new_thing_addr, new_thing_bump) =
        Pubkey::find_program_address(&[b"thing"], &example_account_management::ID);
    assert_eq!(new_thing_bump, 255);

    let mut transaction = Transaction::new_with_payer(
        &[InitThing {
            payer: AccountMeta::new(payer.pubkey(), true),
            new_thing: AccountMeta::new(new_thing_addr, false),
            system_program: AccountMeta::new_readonly(system_program::ID, false),
        }
        .to_instruction(value)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();
    assert!(!program_failed(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 5_915);

    // Check the new_thing account.
    let account_data = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .data;
    let thing_data = Thing::try_deserialize_data(&mut &account_data[..]).unwrap();
    assert_eq!(
        account_data.len(),
        thing_data.try_account_space().unwrap() as usize
    );
    assert_eq!(thing_data, Thing { value });

    // Update.
    let new_value = 420;
    assert_ne!(value, new_value);

    let mut transaction = Transaction::new_with_payer(
        &[UpdateThing {
            thing: AccountMeta::new(new_thing_addr, false),
        }
        .to_instruction(new_value)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();
    assert!(!program_failed(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 808);

    // Check the thing account.
    let account_data = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .data;
    let thing_data = Thing::try_deserialize_data(&mut &account_data[..]).unwrap();
    assert_eq!(
        account_data.len(),
        thing_data.try_account_space().unwrap() as usize
    );
    assert_eq!(thing_data, Thing { value: new_value });

    // Close.
    let beneficiary = Pubkey::new_unique();
    let expected_lamports = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let mut transaction = Transaction::new_with_payer(
        &[CloseThing {
            thing: AccountMeta::new(new_thing_addr, false),
            beneficiary: AccountMeta::new(beneficiary, false),
        }
        .to_instruction()],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();
    assert!(!program_failed(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 1_244);

    let closed_thing = banks_client.get_account(new_thing_addr).await.unwrap();
    assert!(closed_thing.is_none());

    let beneficiary_lamports = banks_client
        .get_account(beneficiary)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(beneficiary_lamports, expected_lamports);
}

#[tokio::test]
async fn test_init_thing_already_having_lamports() {
    let value = 420;

    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "example_account_management",
        example_account_management::ID,
        processor!(example_account_management::entrypoint::process_instruction),
    )
    .start()
    .await;

    let (new_thing_addr, new_thing_bump) =
        Pubkey::find_program_address(&[b"thing"], &example_account_management::ID);
    assert_eq!(new_thing_bump, 255);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::transfer(&payer.pubkey(), &new_thing_addr, 1),
            InitThing {
                payer: AccountMeta::new(payer.pubkey(), true),
                new_thing: AccountMeta::new(new_thing_addr, false),
                system_program: AccountMeta::new_readonly(system_program::ID, false),
            }
            .to_instruction(value),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let tx_meta = banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap()
        .metadata
        .unwrap();
    assert!(!program_failed(&tx_meta.log_messages));
    assert_eq!(tx_meta.compute_units_consumed, 9_297);

    // Check the new_thing account.
    let account_data = banks_client
        .get_account(new_thing_addr)
        .await
        .unwrap()
        .unwrap()
        .data;
    let thing_data = Thing::try_deserialize_data(&mut &account_data[..]).unwrap();
    assert_eq!(
        account_data.len(),
        thing_data.try_account_space().unwrap() as usize
    );
    assert_eq!(thing_data, Thing { value });
}

fn program_failed(log_messages: &Vec<String>) -> bool {
    log_messages
        .iter()
        .filter(|line| {
            line.contains(&format!(
                "Program {} failed",
                example_account_management::ID
            ))
        })
        .peekable()
        .peek()
        .is_some()
}

struct InitThing {
    payer: AccountMeta,
    new_thing: AccountMeta,
    system_program: AccountMeta,
}

impl InitThing {
    fn to_instruction(self, value: u64) -> Instruction {
        let InitThing {
            payer,
            new_thing,
            system_program,
        } = self;

        Instruction {
            program_id: example_account_management::ID,
            accounts: vec![payer, new_thing, system_program],
            data: borsh::to_vec(&ProgramInstruction::InitThing(value)).unwrap(),
        }
    }
}

struct UpdateThing {
    thing: AccountMeta,
}

impl UpdateThing {
    fn to_instruction(self, value: u64) -> Instruction {
        let UpdateThing { thing } = self;

        Instruction {
            program_id: example_account_management::ID,
            accounts: vec![thing],
            data: borsh::to_vec(&ProgramInstruction::UpdateThing(value)).unwrap(),
        }
    }
}

struct CloseThing {
    thing: AccountMeta,
    beneficiary: AccountMeta,
}

impl CloseThing {
    fn to_instruction(self) -> Instruction {
        let CloseThing { thing, beneficiary } = self;

        Instruction {
            program_id: example_account_management::ID,
            accounts: vec![thing, beneficiary],
            data: borsh::to_vec(&ProgramInstruction::CloseThing).unwrap(),
        }
    }
}
