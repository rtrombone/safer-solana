use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::discriminator::Discriminator;
use solana_program::pubkey::Pubkey;
use spl_token_2022::extension::ExtensionType;

pub type Selector = [u8; 4];

#[derive(Debug, PartialEq)]
pub struct ExtensionTypes(pub Vec<ExtensionType>);

impl Eq for ExtensionTypes {}

impl BorshDeserialize for ExtensionTypes {
    #[inline(always)]
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let len = u32::deserialize_reader(reader)? as usize;
        let mut extensions = Vec::with_capacity(len);

        for _ in 0..len {
            let bytes = <[u8; 2]>::deserialize_reader(reader)?;
            let extension = ExtensionType::try_from(&bytes[..]).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid extension type")
            })?;
            extensions.push(extension);
        }

        Ok(Self(extensions))
    }
}

impl BorshSerialize for ExtensionTypes {
    #[inline(always)]
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let Self(extensions) = self;
        (extensions.len() as u32).serialize(writer)?;

        for extension in extensions {
            <[u8; 2]>::from(*extension).serialize(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramInstruction {
    Approve {
        amount: u64,
    },
    Burn {
        amount: u64,
    },
    GetAccountDataSize(ExtensionTypes),
    InitMint {
        decimals: u8,
        freeze_authority: Option<Pubkey>,
    },
    InitTokenAccount {
        owner: Pubkey,
        immutable: bool,
    },
    MintTo {
        amount: u64,
    },
    Revoke,
    Transfer {
        amount: u64,
    },
    TransferChecked {
        amount: u64,
        decimals: u8,
    },
    SuboptimalMintTo {
        amount: u64,
    },
}

impl ProgramInstruction {
    pub const APPROVE: Selector = Discriminator::Sha2(b"ix::approve").to_bytes();
    pub const BURN: Selector = Discriminator::Sha2(b"ix::burn").to_bytes();
    pub const GET_ACCOUNT_DATA_SIZE: Selector =
        Discriminator::Sha2(b"ix::get_account_data_size").to_bytes();
    pub const INIT_MINT: Selector = Discriminator::Sha2(b"ix::init_mint").to_bytes();
    pub const INIT_TOKEN_ACCOUNT: Selector =
        Discriminator::Sha2(b"ix::init_token_account").to_bytes();
    pub const MINT_TO: Selector = Discriminator::Sha2(b"ix::mint_to").to_bytes();
    pub const REVOKE: Selector = Discriminator::Sha2(b"ix::revoke").to_bytes();
    pub const SUBOPTIMAL_MINT_TO: Selector =
        Discriminator::Sha2(b"ix::suboptimal_mint_to").to_bytes();
    pub const TRANSFER: Selector = Discriminator::Sha2(b"ix::transfer").to_bytes();
    pub const TRANSFER_CHECKED: Selector = Discriminator::Sha2(b"ix::transfer_checked").to_bytes();
}

impl BorshDeserialize for ProgramInstruction {
    #[inline(always)]
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        match BorshDeserialize::deserialize_reader(reader)? {
            Self::APPROVE => Ok(Self::Approve {
                amount: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::BURN => Ok(Self::Burn {
                amount: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::GET_ACCOUNT_DATA_SIZE => Ok(Self::GetAccountDataSize(
                BorshDeserialize::deserialize_reader(reader)?,
            )),
            Self::INIT_MINT => Ok(Self::InitMint {
                decimals: BorshDeserialize::deserialize_reader(reader)?,
                freeze_authority: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::INIT_TOKEN_ACCOUNT => Ok(Self::InitTokenAccount {
                owner: BorshDeserialize::deserialize_reader(reader)?,
                immutable: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::MINT_TO => Ok(Self::MintTo {
                amount: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::REVOKE => Ok(Self::Revoke),
            Self::SUBOPTIMAL_MINT_TO => Ok(Self::SuboptimalMintTo {
                amount: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::TRANSFER => Ok(Self::Transfer {
                amount: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::TRANSFER_CHECKED => Ok(Self::TransferChecked {
                amount: BorshDeserialize::deserialize_reader(reader)?,
                decimals: BorshDeserialize::deserialize_reader(reader)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid discriminator",
            )),
        }
    }
}

impl BorshSerialize for ProgramInstruction {
    #[inline(always)]
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Self::Approve { amount } => {
                Self::APPROVE.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Burn { amount } => {
                Self::BURN.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::GetAccountDataSize(extensions) => {
                Self::GET_ACCOUNT_DATA_SIZE.serialize(writer)?;
                extensions.serialize(writer)
            }
            Self::InitMint {
                decimals,
                freeze_authority,
            } => {
                Self::INIT_MINT.serialize(writer)?;
                decimals.serialize(writer)?;
                freeze_authority.serialize(writer)
            }
            Self::InitTokenAccount { owner, immutable } => {
                Self::INIT_TOKEN_ACCOUNT.serialize(writer)?;
                owner.serialize(writer)?;
                immutable.serialize(writer)
            }
            Self::MintTo { amount } => {
                Self::MINT_TO.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Revoke => Self::REVOKE.serialize(writer),
            Self::SuboptimalMintTo { amount } => {
                Self::SUBOPTIMAL_MINT_TO.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Transfer { amount } => {
                Self::TRANSFER.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::TransferChecked { amount, decimals } => {
                Self::TRANSFER_CHECKED.serialize(writer)?;
                amount.serialize(writer)?;
                decimals.serialize(writer)
            }
        }
    }
}
