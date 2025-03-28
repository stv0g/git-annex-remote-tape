use flagset::FlagSet;
use git_annex_remote_tape::tape::{Archive, Drive, Media};
use std::collections::HashMap;
use std::io::{self, stdin};
use std::path::PathBuf;
use std::string::ToString;
use std::{io::Write, result::Result};

use crate::command::Command;
use crate::error::Error;
use crate::extension::Extension;

static TAPE_COST: i64 = 1100;

#[derive(Default)]
pub struct Remote<'a> {
    // Options
    drive_path: Option<PathBuf>,

    // Properties
    uuid: Option<uuid::Uuid>,
    git_dir: Option<String>,
    git_remote_name: Option<String>,
    supported_extensions: Option<FlagSet<Extension>>,

    // State
    drive: Option<Drive>,
    media: Option<Media<'a>>,
    archive: Option<Archive<'a>>,

    prepared: bool,
}

impl<'a> Remote<'a> {
    pub fn new() -> Remote<'a> {
        Remote::default()
    }

    /// Fetch retrieves the necessary information about the remote from git-annex.
    fn fetch(&mut self, initialize: bool) -> Result<(), Error> {
        let drive = self.get_option("drive")?;

        self.drive_path = Some(drive.into());
        self.uuid = Some(self.get_uuid()?);
        self.git_dir = Some(self.get_git_dir()?);

        if let Some(exts) = self.supported_extensions {
            if exts.contains(Extension::GetGitRemoteName) && !initialize {
                self.git_remote_name = Some(self.get_git_remote_name()?);
            }
        }

        Ok(())
    }

    fn init(&mut self) -> Result<(), Error> {
        self.fetch(true)?;

        writeln!(io::stdout(), "INITREMOTE-SUCCESS")?;

        Ok(())
    }

    fn prepare(&mut self) -> Result<(), Error> {
        self.fetch(false)?;

        self.prepared = true;

        writeln!(io::stdout(), "PREPARE-SUCCESS")?;

        Ok(())
    }

    fn extensions(&mut self, arg: Option<&str>) -> Result<(), Error> {
        match arg {
            Some(extensions) => {
                self.supported_extensions = Some(Extension::from_strs(extensions)?);
            }
            None => {}
        }

        writeln!(io::stdout(), "EXTENSIONS INFO")?;
        Ok(())
    }

    fn transfer_store(&self, key: &str, file: &str) -> Result<(), Error> {
        self.set_state(key, format!("block={} offset={}", 1234, 999).as_str())?;

        writeln!(io::stdout(), "TRANSFER-SUCCESS STORE {key}")?;

        Ok(())
    }

    fn transfer_retrieve(&self, key: &str, file: &str) -> Result<(), Error> {
        let error = "Data not available yet";

        writeln!(io::stdout(), "TRANSFER-FAILURE RETRIEVE {} {}", key, error)?;

        Ok(())
    }

    fn transfer(&self, rest: &str) -> Result<(), Error> {
        let parts: Vec<&str> = rest.splitn(3, " ").collect();
        let [direction, key, file] = parts[..] else {
            return Err(Error::InvalidArguments);
        };

        match direction {
            "STORE" => self.transfer_store(key, file),
            "RETRIEVE" => self.transfer_retrieve(key, file),
            _ => Err(Error::InvalidDirection),
        }
    }

    fn check_present(&self, key: &str) -> Result<(), Error> {
        let state = self.get_state(key)?;

        writeln!(
            io::stdout(),
            "CHECKPRESENT-{} {key}",
            match state.as_str() {
                "" => "FAILURE",
                _ => "SUCCESS",
            }
        )?;

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), Error> {
        writeln!(
            io::stdout(),
            "REMOVE-FAILURE {} Droping keys from tapes is currently not supported",
            key
        )?;

        Ok(())
    }

    fn list_configs(&self) -> Result<(), Error> {
        writeln!(
            io::stdout(),
            "CONFIG drive Path of the SCSI tape drive (e.g. /dev/st0)"
        )?;

        writeln!(io::stdout(), "CONFIGEND")?;

        Ok(())
    }

    fn export_supported(&self) -> Result<(), Error> {
        writeln!(io::stdout(), "EXPORTSUPPORTED-FAILURE")?;

        Ok(())
    }

    fn get_cost(&self) -> Result<(), Error> {
        writeln!(io::stdout(), "COST {}", TAPE_COST)?;

        Ok(())
    }

    fn get_ordered(&self) -> Result<(), Error> {
        writeln!(io::stdout(), "ORDERED")?;

        Ok(())
    }

    fn get_availability(&self) -> Result<(), Error> {
        writeln!(io::stdout(), "AVAILABILITY UNAVAILABLE")?;

        Ok(())
    }

    fn get_info(&self) -> Result<(), Error> {
        let mut infos = HashMap::<&'static str, String>::new();

        match &self.drive_path {
            Some(path) => {
                infos.insert("drive", path.display().to_string());
            }
            None => (),
        }

        for (key, value) in infos {
            writeln!(io::stdout(), "INFOFIELD {key}")?;
            writeln!(io::stdout(), "INFOVALUE {value}")?;
        }

        writeln!(io::stdout(), "INFOEND")?;

        Ok(())
    }

    fn get_state(&self, key: &str) -> Result<String, Error> {
        writeln!(io::stdout(), "GETSTATE {key}")?;

        self.read_value()
    }

    fn set_state(&self, key: &str, value: &str) -> Result<(), Error> {
        writeln!(io::stdout(), "SETSTATE {key} {value}")?;

        Ok(())
    }

    fn info(&self, msg: &str) -> Result<(), Error> {
        // INFO is a protocol extension which must only sent after the client has
        // has indicated that it supports it in the EXTENSIONS command.
        if self.prepared {
            writeln!(io::stdout(), "INFO {}", msg)?;
        }

        Ok(())
    }

    fn debug(&self, msg: &str) -> Result<(), Error> {
        // git-annex is not happy when this command is send too early.
        if self.prepared {
            writeln!(io::stdout(), "DEBUG {}", msg)?;
        }

        Ok(())
    }

    fn error(&self, msg: &str) -> Result<(), Error> {
        writeln!(io::stdout(), "ERROR {}", msg)?;

        Ok(())
    }

    fn handle_error(&self, arg: Option<&str>) -> Result<(), Error> {
        Ok(())
    }

    fn get_option(&self, option: &str) -> Result<String, Error> {
        writeln!(io::stdout(), "GETCONFIG {}", option)?;

        self.read_value()
    }

    fn get_uuid(&self) -> Result<uuid::Uuid, Error> {
        let uuid_str = self.get_value("GETUUID")?;

        uuid::Uuid::parse_str(uuid_str.as_str()).map_err(|_| Error::InvalidArguments)
    }

    fn get_git_dir(&self) -> Result<String, Error> {
        self.get_value("GETGITDIR")
    }

    fn get_git_remote_name(&self) -> Result<String, Error> {
        self.get_value("GETGITREMOTENAME")
    }

    fn get_value(&self, cmd: &str) -> Result<String, Error> {
        writeln!(io::stdout(), "{}", cmd)?;

        self.read_value()
    }

    fn read_value(&self) -> Result<String, Error> {
        let line = self.read_line()?;
        let Some((cmd, value)) = line.split_once(" ") else {
            return Err(Error::InvalidArguments);
        };

        if cmd != "VALUE" {
            return Err(Error::InvalidCommand);
        }

        Ok(value.to_string())
    }

    fn process_line(&mut self, line: &str) -> Result<(), Error> {
        let (cmd_str, args) = match line.split_once(" ") {
            Some((cmd, args)) => (cmd, Some(args)),
            None => (line, None),
        };

        let _ = self.debug(format!("Processing line: {}", line).as_str());

        if let Ok(cmd) = Command::from_str(cmd_str) {
            use Command::*;
            match cmd {
                InitRemote => self.init(),
                Extensions => self.extensions(args),
                Prepare => self.prepare(),
                Transfer => self.transfer(args.unwrap()),
                CheckPresent => self.check_present(args.unwrap()),
                Remove => self.remove(args.unwrap()),
                ListConfigs => self.list_configs(),
                ExportSupported => self.export_supported(),
                GetCost => self.get_cost(),
                GetOrdered => self.get_ordered(),
                GetAvailability => self.get_availability(),
                GetInfo => self.get_info(),
            }
        } else {
            writeln!(io::stdout(), "UNSUPPORTED-REQUEST")?;
            Ok(())
        }
    }

    fn read_line(&self) -> Result<String, Error> {
        let mut line = String::new();

        match stdin().read_line(&mut line) {
            Ok(0) => Err(Error::EndOfFile),
            Ok(_) => Ok(line.trim_end_matches("\n").to_string()),

            Err(e) => Err(Error::IO(e)),
        }
    }

    pub fn run(&mut self) {
        writeln!(io::stdout(), "VERSION 2").unwrap();

        loop {
            match self.read_line() {
                Ok(line) => match self.process_line(line.as_str()) {
                    Err(e) => {
                        self.error(format!("Failed to process line: {e:?}").as_str())
                            .unwrap();
                    }
                    Ok(_) => {}
                },
                Err(Error::EndOfFile) => break,
                Err(e) => {
                    self.error(format!("Failed to read line: {e:?}").as_str())
                        .unwrap();
                }
            }
        }
    }
}
