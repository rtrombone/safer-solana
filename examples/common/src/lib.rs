use solana_banks_interface::TransactionMetadata;
use solana_program_test::BanksClient;
use solana_pubkey::Pubkey;
use solana_sdk::{hash::Hash, signer::keypair::Keypair};

pub fn is_program_failure(program_id: &Pubkey, log_messages: &[String]) -> bool {
    log_messages
        .iter()
        .filter(|line| line.contains(&format!("Program {} failed", program_id)))
        .peekable()
        .peek()
        .is_some()
}

pub fn is_compute_units_within(cu_consumed: u64, target_cu: u64, within: u64) -> bool {
    dbg!(cu_consumed, target_cu, within);
    cu_consumed >= target_cu.saturating_sub(within)
        && cu_consumed <= target_cu.saturating_add(within)
}

pub struct TestSuccess {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
    pub tx_meta: TransactionMetadata,
}

pub enum TestResult {
    Fail(TransactionMetadata),
    Success(TestSuccess),
}

impl TestResult {
    pub fn fail(self) -> Option<TransactionMetadata> {
        match self {
            TestResult::Fail(tx_meta) => Some(tx_meta),
            _ => None,
        }
    }

    pub fn success(self) -> Option<TestSuccess> {
        match self {
            TestResult::Success(success) => Some(success),
            _ => None,
        }
    }
}

impl From<TestSuccess> for TestResult {
    fn from(success: TestSuccess) -> Self {
        TestResult::Success(success)
    }
}
