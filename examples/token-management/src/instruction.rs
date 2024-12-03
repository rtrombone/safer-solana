use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::{discriminator::Discriminator, pubkey::Pubkey};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitMintWithExtensionsData {
    pub decimals: u8,
    pub freeze_authority: Option<Pubkey>,
    pub close_authority: bool,
    pub group_pointer: bool,
    pub group_member_pointer: bool,
    pub metadata_pointer: bool,
    pub non_transferable: bool,
    pub permanent_delegate: bool,
    pub transfer_fee: bool,
    pub transfer_hook: bool,
    pub confidential_transfer: bool,
    pub confidential_transfer_fee: bool,
}

impl InitMintWithExtensionsData {
    const CLOSE_AUTHORITY_BIT: u16 = 0b1;
    const GROUP_POINTER_BIT: u16 = 0b10;
    const GROUP_MEMBER_POINTER_BIT: u16 = 0b100;
    const METADATA_POINTER_BIT: u16 = 0b1000;
    const NON_TRANSFERABLE_BIT: u16 = 0b1_0000;
    const PERMANENT_DELEGATE_BIT: u16 = 0b10_0000;
    const TRANSFER_FEE_BIT: u16 = 0b100_0000;
    const TRANSFER_HOOK_BIT: u16 = 0b1000_0000;
    const CONFIDENTIAL_TRANSFER_BIT: u16 = 0b1_0000_0000;
    const CONFIDENTIAL_TRANSFER_FEE_BIT: u16 = 0b10_0000_0000;
}

impl BorshDeserialize for InitMintWithExtensionsData {
    #[inline(always)]
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let decimals = BorshDeserialize::deserialize_reader(reader)?;
        let freeze_authority = BorshDeserialize::deserialize_reader(reader)?;

        let extensions_flags = u16::deserialize_reader(reader)?;
        let close_authority = (extensions_flags & Self::CLOSE_AUTHORITY_BIT) != 0;
        let group_pointer = (extensions_flags & Self::GROUP_POINTER_BIT) != 0;
        let group_member_pointer = (extensions_flags & Self::GROUP_MEMBER_POINTER_BIT) != 0;
        let metadata_pointer = (extensions_flags & Self::METADATA_POINTER_BIT) != 0;
        let non_transferable = (extensions_flags & Self::NON_TRANSFERABLE_BIT) != 0;
        let permanent_delegate = (extensions_flags & Self::PERMANENT_DELEGATE_BIT) != 0;
        let transfer_fee = (extensions_flags & Self::TRANSFER_FEE_BIT) != 0;
        let transfer_hook = (extensions_flags & Self::TRANSFER_HOOK_BIT) != 0;
        let confidential_transfer = (extensions_flags & Self::CONFIDENTIAL_TRANSFER_BIT) != 0;
        let confidential_transfer_fee =
            (extensions_flags & Self::CONFIDENTIAL_TRANSFER_FEE_BIT) != 0;

        Ok(Self {
            decimals,
            freeze_authority,
            close_authority,
            group_pointer,
            group_member_pointer,
            metadata_pointer,
            non_transferable,
            permanent_delegate,
            transfer_fee,
            transfer_hook,
            confidential_transfer,
            confidential_transfer_fee,
        })
    }
}

impl BorshSerialize for InitMintWithExtensionsData {
    #[inline(always)]
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.decimals.serialize(writer)?;
        self.freeze_authority.serialize(writer)?;

        let mut extensions_flags = u16::default();
        if self.close_authority {
            extensions_flags |= Self::CLOSE_AUTHORITY_BIT;
        }
        if self.group_pointer {
            extensions_flags |= Self::GROUP_POINTER_BIT;
        }
        if self.group_member_pointer {
            extensions_flags |= Self::GROUP_MEMBER_POINTER_BIT;
        }
        if self.metadata_pointer {
            extensions_flags |= Self::METADATA_POINTER_BIT;
        }
        if self.non_transferable {
            extensions_flags |= Self::NON_TRANSFERABLE_BIT;
        }
        if self.permanent_delegate {
            extensions_flags |= Self::PERMANENT_DELEGATE_BIT;
        }
        if self.transfer_fee {
            extensions_flags |= Self::TRANSFER_FEE_BIT;
        }
        if self.transfer_hook {
            extensions_flags |= Self::TRANSFER_HOOK_BIT;
        }
        if self.confidential_transfer {
            extensions_flags |= Self::CONFIDENTIAL_TRANSFER_BIT;
        }
        if self.confidential_transfer_fee {
            extensions_flags |= Self::CONFIDENTIAL_TRANSFER_FEE_BIT;
        }
        extensions_flags.serialize(writer)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramInstruction {
    Approve(u64),
    Burn(u64),
    GetAccountDataSize(ExtensionTypes),
    InitAta(bool),
    InitMint(InitMintWithExtensionsData),
    InitTokenAccount {
        owner: Pubkey,
        immutable_owner: bool,
    },
    MintTo(u64),
    Revoke,
    Transfer(u64),
    TransferChecked {
        amount: u64,
        decimals: u8,
    },
    SuboptimalMintTo(u64),
}

impl ProgramInstruction {
    pub const APPROVE: Selector = Discriminator::Sha2(b"ix::approve").to_bytes();
    pub const BURN: Selector = Discriminator::Sha2(b"ix::burn").to_bytes();
    pub const GET_ACCOUNT_DATA_SIZE: Selector =
        Discriminator::Sha2(b"ix::get_account_data_size").to_bytes();
    pub const INIT_ATA: Selector = Discriminator::Sha2(b"ix::init_ata").to_bytes();
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
            Self::APPROVE => Ok(Self::Approve(BorshDeserialize::deserialize_reader(reader)?)),
            Self::BURN => Ok(Self::Burn(BorshDeserialize::deserialize_reader(reader)?)),
            Self::GET_ACCOUNT_DATA_SIZE => Ok(Self::GetAccountDataSize(
                BorshDeserialize::deserialize_reader(reader)?,
            )),
            Self::INIT_ATA => Ok(Self::InitAta(BorshDeserialize::deserialize_reader(reader)?)),
            Self::INIT_MINT => Ok(Self::InitMint(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
            Self::INIT_TOKEN_ACCOUNT => Ok(Self::InitTokenAccount {
                owner: BorshDeserialize::deserialize_reader(reader)?,
                immutable_owner: BorshDeserialize::deserialize_reader(reader)?,
            }),
            Self::MINT_TO => Ok(Self::MintTo(BorshDeserialize::deserialize_reader(reader)?)),
            Self::REVOKE => Ok(Self::Revoke),
            Self::SUBOPTIMAL_MINT_TO => Ok(Self::SuboptimalMintTo(
                BorshDeserialize::deserialize_reader(reader)?,
            )),
            Self::TRANSFER => Ok(Self::Transfer(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
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
            Self::Approve(amount) => {
                Self::APPROVE.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Burn(amount) => {
                Self::BURN.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::GetAccountDataSize(extensions) => {
                Self::GET_ACCOUNT_DATA_SIZE.serialize(writer)?;
                extensions.serialize(writer)
            }
            Self::InitAta(idempotent) => {
                Self::INIT_ATA.serialize(writer)?;
                idempotent.serialize(writer)
            }
            Self::InitMint(data) => {
                Self::INIT_MINT.serialize(writer)?;
                data.serialize(writer)
            }
            Self::InitTokenAccount {
                owner,
                immutable_owner,
            } => {
                Self::INIT_TOKEN_ACCOUNT.serialize(writer)?;
                owner.serialize(writer)?;
                immutable_owner.serialize(writer)
            }
            Self::MintTo(amount) => {
                Self::MINT_TO.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Revoke => Self::REVOKE.serialize(writer),
            Self::SuboptimalMintTo(amount) => {
                Self::SUBOPTIMAL_MINT_TO.serialize(writer)?;
                amount.serialize(writer)
            }
            Self::Transfer(amount) => {
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
