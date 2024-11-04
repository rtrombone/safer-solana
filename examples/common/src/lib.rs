use solana_program::pubkey::Pubkey;

pub fn program_failed(program_id: &Pubkey, log_messages: &[String]) -> bool {
    log_messages
        .iter()
        .filter(|line| line.contains(&format!("Program {} failed", program_id)))
        .peekable()
        .peek()
        .is_some()
}
