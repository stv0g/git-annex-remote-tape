use std::str::FromStr;

use crate::error::Error;

pub enum Command {
    InitRemote,
    Extensions,
    Prepare,
    Transfer,
    CheckPresent,
    Remove,
    ListConfigs,
    ExportSupported,
    GetCost,
    GetOrdered,
    GetAvailability,
    GetInfo,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "INITREMOTE" => Ok(Self::InitRemote),
            "EXTENSIONS" => Ok(Self::Extensions),
            "PREPARE" => Ok(Self::Prepare),
            "TRANSFER" => Ok(Self::Transfer),
            "CHECKPRESENT" => Ok(Self::CheckPresent),
            "REMOVE" => Ok(Self::Remove),
            "LISTCONFIGS" => Ok(Self::ListConfigs),
            "EXPORTSUPPORTED" => Ok(Self::ExportSupported),
            "GETCOST" => Ok(Self::GetCost),
            "GETORDERED" => Ok(Self::GetOrdered),
            "GETAVAILABILITY" => Ok(Self::GetAvailability),
            "GETINFO" => Ok(Self::GetInfo),
            _ => Err(Error::InvalidCommand),
        }
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Self::InitRemote => "INITREMOTE",
            Self::Extensions => "EXTENSIONS",
            Self::Prepare => "PREPARE",
            Self::Transfer => "TRANSFER",
            Self::CheckPresent => "CHECKPRESENT",
            Self::Remove => "REMOVE",
            Self::ListConfigs => "LISTCONFIGS",
            Self::ExportSupported => "EXPORTSUPPORTED",
            Self::GetCost => "GETCOST",
            Self::GetOrdered => "GETORDERED",
            Self::GetAvailability => "GETAVAILABILITY",
            Self::GetInfo => "GETINFO",
        }
        .to_string()
    }
}
