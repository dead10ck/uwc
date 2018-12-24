#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

#[macro_use]
extern crate failure;

extern crate env_logger;
extern crate itertools;
extern crate tabwriter;
extern crate unicode_segmentation;

extern crate rayon;
use rayon::prelude::*;

mod counter;
mod error;
mod input;
mod opt;
mod ubufreader;

use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::{self, BufReader, Write};
use std::iter::IntoIterator;
use std::sync::{Arc, Mutex};

use failure::Error;
use itertools::Itertools;
use structopt::StructOpt;
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
    mut writer: W,
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

fn sum_counts<'a, I>(counts: I) -> BTreeMap<Counter, usize>
where
    I: IntoIterator<Item = &'a BTreeMap<Counter, usize>>,
{
    let mut totals = BTreeMap::new();

    for counts in counts {
        for (counter, count) in counts.iter() {
            let c = totals.entry(*counter).or_insert(0);
            *c += count;
        }
    }

    totals
}

/// The return type indicates error conditions. In some error cases, it will just
/// print the error and continue counting (e.g., if the user passes a directory
/// as input). A return value of Ok(true) indicates that the run was successful
/// with no errors; Ok(false) indicates that there were errors, but not fatal
/// to the `run` function. A return value of `Err` indicates a fatal error that
/// needed to exit immediately, e.g., writing to stdout failed.
fn run() -> Result<bool, Error> {
    let opts = Opt::from_args();

    debug!("opts: {:?}", opts);

    let counters = opts.get_counters();
    let keep_newlines = opts.should_keep_newlines();
    let mode = opts.mode;

    let mut counts: BTreeMap<String, Counted> = opts
        .files
        .into_iter()
        .map(|fname| {
            (
                fname,
                counters.iter().map(|c| (*c, 0usize)).collect::<Counted>(),
            )
        })
        .collect();

    let stdout = io::stdout();

    let writer: Arc<Mutex<Write + Send + Sync>> = if opts.no_elastic {
        Arc::new(Mutex::new(stdout))
    } else {
        Arc::new(Mutex::new(TabWriter::new(stdout)))
    };

    if !opts.no_header {
        write_header(&mut *writer.lock().unwrap(), &counters)?;
    }

    let success = counts
        .par_iter_mut()
        .map(|(file_name, mut file_counts)| {
            info!("Counting file: {}", file_name);

            let mut success = true;

            let input = match Input::new(&file_name) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("{}: {}", &file_name, e);
                    return Err(Error::from(e));
                }
            };

            let mut reader = BufReader::new(input);
            let chunks = UStrChunksIter::new(&mut reader, keep_newlines);

            for chunk in &chunks.chunks(10_000) {
                let chunk: Vec<_> = chunk.collect();

                let (chunk_success, line_counts) = chunk
                    .into_par_iter()
                    .enumerate()
                    .map(|(line_no, line)| {
                        let line_no = line_no + 1; // to start at 1
                        let line = match line {
                            Ok(l) => l,
                            Err(e) => {
                                eprintln!("{}:{}: {}", file_name, line_no, e);
                                return Ok((false, BTreeMap::new()));
                            }
                        };

                        debug!("line: {:?}", line);

                        let cur_counts = counter::count(&counters, &line);

                        if mode == CountMode::Line {
                            let name = file_name_with_line(&file_name, line_no);
                            write_counts(&mut *writer.lock().unwrap(), &cur_counts, Some(&name))?;
                        }

                        Ok((true, cur_counts))
                    })
                    // sum up the counts for each line into the total counts for
                    // the file
                    .reduce(
                        || Ok((true, Counted::new())),
                        |mut acc: Result<_, Error>, r: Result<_, Error>| {
                            if r.is_err() {
                                return r;
                            }

                            match acc {
                                Err(e) => return Err(e),
                                Ok(ref mut acc_counts_success) => {
                                    // already guaranteed to be ok by the check above
                                    let (mut r_success, mut r_current) = r.unwrap();
                                    let &mut (ref mut acc_success, ref mut acc_counts) =
                                        acc_counts_success;

                                    for (ctr, total) in r_current {
                                        let entry = acc_counts.entry(ctr).or_insert(0);
                                        *entry += total;
                                    }

                                    *acc_success &= r_success;
                                }
                            }

                            acc
                        },
                    )?;

                counter::sum_counts(&mut file_counts, &line_counts);
                success &= chunk_success;
            }

            match mode {
                CountMode::File => {
                    write_counts(&mut *writer.lock().unwrap(), &file_counts, Some(&file_name))?
                }
                CountMode::Line => {
                    let name = file_name_with_line(&file_name, TOTAL);
                    write_counts(&mut *writer.lock().unwrap(), &file_counts, Some(&name))?
                }
            }

            Ok(success)
        })
        .reduce(
            || Ok(true),
            |acc_result, success_result| {
                let acc = acc_result?;
                let success = success_result?;
                Ok(acc && success)
            },
        )?;

    info!("final_counts: {:?}", counts);

    if mode == CountMode::File && counts.len() > 1 {
        let totals = sum_counts(counts.values());
        write_counts(&mut *writer.lock().unwrap(), &totals, Some(TOTAL))?;
    }

    writer.lock().unwrap().flush()?;

    Ok(success)
}
