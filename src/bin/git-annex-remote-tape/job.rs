use crate::cli::JobCommand;

pub fn run(command: JobCommand) {
    match command {
        JobCommand::Info { job_id } => unimplemented!(),
        JobCommand::List {} => unimplemented!(),
        JobCommand::Start { job_id } => unimplemented!(),
        JobCommand::Drop { job_id } => unimplemented!(),
    }
}
