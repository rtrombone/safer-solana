use solana_program::pubkey::Pubkey;

pub trait DeriveAddress {
    type Seeds<'a>;

    /// Using [Pubkey::find_program_address], implement a custom address finder for a PDA.
    fn find_program_address(seeds: Self::Seeds<'_>) -> (Pubkey, u8);

    /// Using [Pubkey::create_program_address], implement a custom address creator for a PDA.
    fn create_program_address(seeds: Self::Seeds<'_>, bump_seed: u8) -> Option<Pubkey>;
}
