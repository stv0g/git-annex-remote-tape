#![allow(dead_code)]

use git_annex_remote_tape::{mt, mtio};
use std::{env, path::PathBuf, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    println!("File path: {:?}", file_path);

    let tape = mt::MagneticTape::new(&file_path).unwrap();

    let opts = tape.get_options().unwrap();
    println!("Current options: {:?}", opts);

    if !opts.contains(mtio::SetDrvBufferOptions::MT_ST_SCSI2LOGICAL) {
        tape.add_options(mtio::SetDrvBufferOptions::MT_ST_SCSI2LOGICAL)
        .unwrap();
    }

    println!("Current block number: {}", tape.get_position().unwrap());
    println!("Current status: {:?}", tape.get_status().unwrap());
    println!("Current position: {}", tape.get_position().unwrap());



    // tape.rewind().unwrap();

    // tape.write_block("Hello world 1".as_bytes()).unwrap();
    // tape.weof(1).unwrap();

    // tape.write_block("Hello world 2".as_bytes()).unwrap();
    // tape.weof(1).unwrap();

    // tape.write_block("Hello world 3".as_bytes()).unwrap();
    // tape.weof(1).unwrap();

    // tape.rewind().unwrap();

    // let mut block = [0u8; 1024];
    // tape.read_block(&mut block).unwrap();

    // let block_str = std::str::from_utf8(&block).unwrap();
    // println!("Read block as string: {}", block_str);

    // println!("Current block number: {}", tape.get_position().unwrap());
}
