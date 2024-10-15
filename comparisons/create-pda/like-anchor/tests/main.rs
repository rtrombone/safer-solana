use create_pda_common::InitThingTest;
use solana_program_test::{processor, tokio, ProgramTest};

#[tokio::test]
async fn test_init_thing() {
    let tx_meta = create_pda_common::init_thing_for_test(set_up().await).await;
    assert_eq!(tx_meta.compute_units_consumed, 5_868);
}

#[tokio::test]
async fn test_init_thing_already_having_lamports() {
    let tx_meta =
        create_pda_common::init_thing_already_having_lamports_for_test(set_up().await).await;

    // This includes lamports transfer.
    assert_eq!(tx_meta.compute_units_consumed, 9_251);
}

async fn set_up() -> InitThingTest {
    let (banks_client, payer, recent_blockhash) = ProgramTest::new(
        "create_pda_like_anchor",
        create_pda_like_anchor::ID,
        processor!(create_pda_like_anchor::process_instruction),
    )
    .start()
    .await;

    InitThingTest {
        banks_client,
        payer,
        recent_blockhash,
        program_id: create_pda_like_anchor::ID,
    }
}
