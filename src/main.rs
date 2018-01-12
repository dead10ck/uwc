#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate env_logger;
extern crate unicode_segmentation;
extern crate tabwriter;

mod input;
mod counter;
mod opt;
mod ubufreader;
mod error;

use std::collections::BTreeMap;
use std::io::{self, BufReader, Write};
use std::iter::IntoIterator;
use std::fmt::Display;

use structopt::StructOpt;
use failure::Error;
use tabwriter::TabWriter;

use counter::{Counted, Counter};
use input::Input;
use opt::{CountMode, Opt};
use ubufreader::UStrChunksIter;

const TOTAL: &'static str = "total";

fn main() {
    env_logger::init().unwrap();

    let run_result = run();

    match run_result {
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
        Ok(success) if success == false => std::process::exit(2),
        _ => {}
    }
}

fn write_counts<W: Write>(
    writer: &mut W,
    counts: &BTreeMap<Counter, usize>,
    title: Option<&str>,
) -> Result<(), Error> {
    let mut out_str = String::new();

    for count in counts.values() {
        out_str.push_str(&count.to_string());
        out_str.push_str("\t");
    }

    // remove the trailing tab
    out_str.pop();

    if let Some(name) = title {
        out_str.push_str("\t");
        out_str.push_str(name);
    }

    out_str.push_str("\n");

    Ok(writer.write_all(&out_str.as_bytes())?)
}

/// Construct the "file name" to display for line mode.
fn file_name_with_line<D: Display>(fname: &str, thing: D) -> String {
    format!("{}:{}", fname, thing)
}

/// Write the header that displays counter names in columns.
fn write_header<'a, W, I>(mut writer: W, counters: I) -> Result<(), Error>
where
    W: Write,
    I: IntoIterator<Item = &'a Counter>,
{
    let mut out_str = String::new();

    for counter in counters.into_iter() {
        out_str.push_str(&counter.to_string());
        out_str.push_str("\t");
    }

    out_str.push_str("filename\n");

    Ok(writer.write_all(&out_str.as_bytes())?)
}

/// The return type indicates error conditions. In some error cases, it will just
/// print the error and continue counting (e.g., if the user passes a directory
/// as input). A return value of Ok(true) indicates that the run was successful
/// with no errors; Ok(false) indicates that there were errors, but not fatal
/// to the `run` function. A return value of `Err` indicates a fatal error that
/// needed to exit immediately, e.g., writing to stdout failed.
fn run() -> Result<bool, Error> {
    let mut success = true;
    let opts = Opt::from_args();

    debug!("opts: {:?}", opts);

    let counters = opts.get_counters();

    let mut counts: BTreeMap<String, Counted> = opts.files
        .into_iter()
        .map(|fname| {
            (
                fname,
                counters.iter().map(|c| (*c, 0usize)).collect::<Counted>(),
            )
        })
        .collect();

    let stdout = io::stdout();
    let handle = stdout.lock();

    let mut writer : Box<Write> = if opts.no_elastic {
        Box::new(handle)
    } else {
        Box::new(TabWriter::new(handle))
    };

    if !opts.no_header {
        write_header(&mut writer, &counters)?;
    }

    for (file_name, file_counts) in &mut counts {
        info!("Counting file: {}", file_name);

        let input = match Input::new(file_name) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{}: {}", file_name, e);
                success = false;
                continue;
            }
        };

        let mut reader = BufReader::new(input);
        let chunks = UStrChunksIter::new(&mut reader);

        for (line_no, line) in chunks.enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    success = false;
                    continue;
                }
            };

            debug!("line: {:?}", line);

            let cur_counts = counter::count(&counters, &line);

            if opts.mode == CountMode::Line {
                let name = file_name_with_line(file_name, line_no);
                write_counts(&mut writer, &cur_counts, Some(&name))?;
            }

            for (counter, cur_count) in cur_counts {
                // Unwrap is ok since this map is constructed with the counters.
                let count = file_counts.get_mut(&counter).unwrap();
                *count += cur_count;
            }
        }

        match opts.mode {
            CountMode::File => write_counts(&mut writer, &file_counts, Some(file_name))?,
            CountMode::Line => {
                let name = file_name_with_line(file_name, TOTAL);
                write_counts(&mut writer, &file_counts, Some(&name))?;
            }
        }
    }

    info!("final_counts: {:?}", counts);

    if opts.mode == CountMode::File && counts.len() > 1 {
        let mut totals = BTreeMap::new();

        for file_counts in counts.values() {
            for (counter, count) in file_counts.iter() {
                let c = totals.entry(*counter).or_insert(0);
                *c += count;
            }
        }

        write_counts(&mut writer, &totals, Some(TOTAL))?;
    }

    writer.flush()?;

    Ok(success)
}
