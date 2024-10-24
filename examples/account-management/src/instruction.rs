use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::discriminator::Discriminator;

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramInstruction {
    InitThing(u64),
    UpdateThing(u64),
    CloseThing,
}

impl ProgramInstruction {
    pub const INIT_THING: [u8; 4] = Discriminator::Sha2(b"ix::init_thing").to_bytes();
    pub const UPDATE_THING: [u8; 4] = Discriminator::Sha2(b"ix::update_thing").to_bytes();
    pub const CLOSE_THING: [u8; 4] = Discriminator::Sha2(b"ix::close_thing").to_bytes();
}

impl BorshDeserialize for ProgramInstruction {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        match BorshDeserialize::deserialize_reader(reader)? {
            Self::INIT_THING => Ok(Self::InitThing(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
            Self::UPDATE_THING => Ok(Self::UpdateThing(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
            Self::CLOSE_THING => Ok(Self::CloseThing),
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
            Self::InitThing(value) => {
                Self::INIT_THING.serialize(writer)?;
                value.serialize(writer)
            }
            Self::UpdateThing(value) => {
                Self::UPDATE_THING.serialize(writer)?;
                value.serialize(writer)
            }
            Self::CloseThing => Self::CLOSE_THING.serialize(writer),
        }
    }
}
