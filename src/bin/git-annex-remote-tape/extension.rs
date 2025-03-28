use flagset::{flags, FlagSet};

use crate::error::Error;

flags! {
    pub enum Extension: u32 {
        Info,
        Async,
        GetGitRemoteName,
        UnavailableResponse,
    }
}

impl Extension {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "INFO" => Ok(Self::Info),
            "ASYNC" => Ok(Self::Async),
            "GETGITREMOTENAME" => Ok(Self::GetGitRemoteName),
            "UNAVAILABLERESPONSE" => Ok(Self::UnavailableResponse),
            _ => Err(Error::InvalidArguments),
        }
    }

    fn from_strs(s: &str) -> Result<FlagSet<Self>, Error> {
        let mut es = FlagSet::<Self>::new_truncated(0);

        for p in s.split_ascii_whitespace() {
            es |= Self::from_str(p)?;
        }

        Ok(es)
    }
}
