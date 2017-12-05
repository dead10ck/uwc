#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate structopt_derive;
extern crate structopt;

#[macro_use]
extern crate failure_derive;
extern crate failure;

extern crate env_logger;
extern crate unicode_segmentation;

mod input;
mod counter;
mod opt;
mod ubufreader;
mod error;

use std::collections::HashMap;
use std::io;
use std::io::{BufRead, BufReader};

use structopt::StructOpt;

use counter::Counter;
use input::Input;
use opt::Opt;

fn main() {
    env_logger::init().unwrap();

    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let opts = Opt::from_args();

    debug!("opts: {:?}", opts);

    let counters = opts.get_counters();
    let mut final_counts: HashMap<Counter, usize> = counters.iter().map(|c| (*c, 0)).collect();

    for file_name in opts.files {
        let input = Input::new(file_name)?;
        let reader = BufReader::new(input);

        for line in reader.lines() {
            let line = line?;
            debug!("line: {:?}", line);

            let cur_counts = counter::count(&counters, &line);

            for (counter, cur_count) in cur_counts {
                // Unwrap is ok since this map is constructed with the counters.
                let count = final_counts.get_mut(&counter).unwrap();
                *count += cur_count;
            }
        }
    }

    info!("final_counts: {:?}", final_counts);

    Ok(())
}
