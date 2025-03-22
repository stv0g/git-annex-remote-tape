use std::{io::Write, result::Result};
use std::fs::File;
use std::fmt::{Display, Formatter};
use std::fmt;
use clap::Parser;
use std::io::{self};

static STATE_DIR = "/Users/stv0g/.local/var/lib/git-annex/remotes/tape"

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() {
    let cli = Cli::parse();

    let mut remote = Remote::new();

    loop {
        remote.run();
    }
}

#[derive(Debug)]
struct Error {
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            message: error.to_string(),
        }
    }
}

#[derive(Debug)]
struct Tape {
    ID: String,
    Generation: u32,
}

#[derive(Debug, Default)]
struct Options {
    drive: Option<String>,
    tape: Tape,
}

struct Remote{
    stdin: io::Stdin,
    stdout: io::Stdout,

    opts: Options,

    f: Option<File>,
}

impl Remote {
    fn new() -> Remote {
        Remote {
            stdin: io::stdin(),
            stdout: io::stdout(),
            opts: Options::default(),
            f: None,
        }
    }

    fn init_remote(&mut self) -> Result<(), Error> {
        self.write_line("INITREMOTE-SUCCESS".to_string())?;

        Ok(())
    }

    fn extensions(&mut self) -> Result<(), Error> {
        self.write_line("EXTENSIONS INFO".to_string())?;

        Ok(())
    }

    fn prepare(&mut self) -> Result<(), Error> {
        let drive = self.get_option("drive")?;

        self.opts.drive = Some(drive.clone());

        self.f = Some(File::open(drive)?);

        self.write_line("PREPARE-SUCCESS".to_string())?;

        Ok(())
    }

    fn transfer_store(&mut self, key: &str, file: &str)-> Result<(), Error>  {
        self.info(format!("store: {} {}", key, file).as_str())?;

        Ok(())
    }

    fn transfer_retrieve(&mut self, key: &str, file: &str)-> Result<(), Error>  {
        self.info(format!("retrieve: {} {}", key, file).as_str())?;

        let error = "Data not available yet";

        self.write_line(format!("TRANSFER-FAILURE RETRIEVE {} {}", key, error))?;

        Ok(())
    }

    fn transfer(&mut self, rest: &str) -> Result<(), Error> {
        let parts: Vec<&str> = rest.splitn(3, " ").collect();
        let [direction, key, file] = parts[..] else {
            return Err(Error {
                message: "Invalid number of arguments".to_string(),
            });
        };

        match direction {
            "STORE" =>  self.transfer_store(key, file)
            "RETRIEVE" =>  self.transfer_retrieve(key, file)
            _ =>
                Err(Error {
                    message: "Invalid direction".to_string(),
                })
        }
    }

    fn check_present(&mut self, key: &str) -> Result<(), Error> {
        self.info(format!("check present: {}", key).as_str())
    }

    fn remove(&mut self, key: &str) -> Result<(), Error> {
        self.info(format!("remove: {}", key).as_str())?;

        self.write_line(format!("REMOVE-FAILURE {} Droping keys from tapes is currently not supported", key))
    }

    fn list_configs(&mut self) -> Result<(), Error>{
        self.write_line("CONFIG drive Path of the SCSI tape drive (e.g. /dev/st0)".to_string())?;
        self.write_line("CONFIGEND".to_string())?;

        Ok(())
    }

    fn get_cost(&mut self) -> Result<(), Error> {
        self.write_line("COST 1100".to_string())?;

        Ok(())
    }

    fn get_ordered(&mut self)-> Result<(), Error> {
        self.write_line("ORDERED".to_string())?;

        Ok(())
    }

    fn get_availability(&mut self)-> Result<(), Error> {
        self.write_line("AVAILABILITY UNAVAILABLE".to_string())?;

        Ok(())
    }

    fn get_info(&self) -> Result<(), Error>{
        Ok(())
    }

    fn info(&mut self, msg: String) -> Result<(), Error>{
        self.write_line(format!("INFO {}", msg))?;

        Ok(())
    }

    fn debug(&mut self, msg: String) -> Result<(), Error>{
        self.write_line(format!("DEBUG {}", msg))?;

        Ok(())
    }

    fn error(&mut self, msg: String) -> Result<(), Error>{
        self.write_line(format!("ERROR {}", msg))?;

        Ok(())
    }

    fn get_option(&mut self, option: String) -> Result<String, Error> {
        self.write_line(format!("GETCONFIG {}", option))?;
        self.read_value()
    }

    fn get_uuid(&mut self, option: String) -> Result<String, Error> {
        self.write_line(format!("GETUUID {}", option))?;
        self.read_value()
    }

    fn get_git_dir(&mut self, option: String) -> Result<String, Error> {
        self.write_line(format!("GETGITDIR {}", option))?;
        self.read_value()
    }

    fn get_git_remote_name(&mut self, option: String) -> Result<String, Error> {
        self.write_line(format!("GETGITREMOTENAME {}", option))?;
        self.read_value()
    }

    fn read_value(&mut self) -> Result<String, Error> {
        let line = self.read_line()?;

        let parts: Vec<&str> = line.splitn(2, " ").collect();
        let [cmd, value] = parts[..] else {
            return Err(Error {
                message: "Invalid number of arguments".to_string(),
            });
        };

        if cmd != "VALUE" {
            return Err(Error {
                message: "Invalid command".to_string(),
            });
        }

        Ok(value.to_string())
    }

    fn read_line(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        self.stdin.read_line(&mut line)?;

        Ok(line.trim_end().to_string())
    }

    fn write_line(&mut self, msg: String) -> io::Result<usize> {
        self.stdout.write(format!("{}\n", msg).as_bytes())
    }

    fn process_line(&mut self, line: String) -> Result<(), Error> {        
        let (cmd, args: Option<String>) = match line.split_once(" ") {
            Some((cmd, args)) => (cmd, Some(args)),
            None => (line, None)
        };
    
        match cmd {
            "INITREMOTE" =>  self.init_remote()
            "EXTENSIONS" =>  self.extensions()
            "PREPARE" =>  self.prepare()
            "TRANSFER" =>  self.transfer(args)
            "CHECKPRESENT" =>  self.check_present(args)
            "REMOVE" =>  self.remove(args)
            "LISTCONFIGS" =>  self.list_configs()
            "GETCOST" =>  self.get_cost()
            "GETORDERED" =>  self.get_ordered()
            "GETAVAILABILITY" =>  self.get_availability()
            "GETINFO" =>  self.get_info()
            _ => 
                Err(Error {
                    message: "Invalid command".to_string(),
                }),
        }
    }

    fn run(&mut self) {
        self.write_line("VERSION 2").unwrap();

        loop {
            match self.read_line() {
                Ok(line) => {
                    match self.process_line(line) {
                        Ok(_) => {}
                        Err(e) => {
                            self.error(e.message.as_str()).unwrap();
                        }
                    }
                }
                Err(e) => {
                    self.error(e.to_string().as_str()).unwrap();
                }
            }
        }
    }
    
}


