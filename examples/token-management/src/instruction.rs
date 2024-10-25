use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::discriminator::Discriminator;
use solana_program::pubkey::Pubkey;

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramInstruction {
    InitMint {
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
    },
}

impl ProgramInstruction {
    pub const INIT_MINT: [u8; 4] = Discriminator::Sha2(b"ix::init_mint").to_bytes();
}

impl BorshDeserialize for ProgramInstruction {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        match BorshDeserialize::deserialize_reader(reader)? {
            Self::INIT_MINT => Ok(Self::InitMint {
                decimals: BorshDeserialize::deserialize_reader(reader)?,
                mint_authority: BorshDeserialize::deserialize_reader(reader)?,
                freeze_authority: BorshDeserialize::deserialize_reader(reader)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid discriminator",
            )),
        }
    }
}

impl BorshSerialize for ProgramInstruction {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Self::InitMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                Self::INIT_MINT.serialize(writer)?;
                decimals.serialize(writer)?;
                mint_authority.serialize(writer)?;
                freeze_authority.serialize(writer)
            }
        }
    }
}
