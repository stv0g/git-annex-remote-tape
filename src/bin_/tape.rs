use git_annex_remote_tape::mt;
use std::{env, path::PathBuf, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);

    let tape = mt::Tape::new(&file_path).unwrap();

    let result = tape.get_position();

    match result {
        Ok(pos) => {
            println!("Current block number: {}", pos);
        }
        Err(err) => {
            eprintln!("Error performing MTIOCPOS ioctl: {}", err);
            process::exit(1);
        }
    }
}
