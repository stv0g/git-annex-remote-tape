use crate::cli::TapeCommand;

pub fn run(command: TapeCommand) {
    match command {
        TapeCommand::Init {} => unimplemented!(),
        TapeCommand::Erase { secure } => unimplemented!(),
        TapeCommand::Info {} => unimplemented!(),
    }
}
