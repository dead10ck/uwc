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

use std::collections::BTreeMap;
use std::io::{ self, Write, BufReader };

use structopt::StructOpt;
use failure::Error;

use counter::Counter;
use input::Input;
use opt::Opt;
use ubufreader::UStrChunksIter;

fn main() {
    env_logger::init().unwrap();

    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn write_counts<W: Write>(writer: &mut W, counts: &BTreeMap<Counter, usize>, fname: Option<&str>) -> Result<(), Error> {
    let mut out_str = String::new();

    let num_counts = counts.values().len();

    for (i, count) in counts.values().enumerate() {
        out_str.push_str(&count.to_string());

        if i <= num_counts - 1 {
            out_str.push_str("\t");
        }
    }

    if let Some(name) = fname {
        out_str.push_str("\t");
        out_str.push_str(name);
    }

    out_str.push_str("\n");

    Ok(writer.write_all(&out_str.as_bytes())?)
}

fn run() -> Result<(), Error> {
    let opts = Opt::from_args();

    debug!("opts: {:?}", opts);

    let counters = opts.get_counters();
    let mut final_counts: BTreeMap<Counter, usize> = counters.iter().map(|c| (*c, 0)).collect();

    for file_name in opts.files {
        let input = Input::new(file_name)?;
        let mut reader = BufReader::new(input);
        let chunks = UStrChunksIter::new(&mut reader);

        for line in chunks {
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

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    write_counts(&mut handle, &final_counts, None)?;

    Ok(())
}
