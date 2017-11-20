#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate env_logger;
extern crate clap;
extern crate unicode_segmentation;

mod input;
mod counter;

use std::io;
use std::io::{BufRead, BufReader};

use clap::{Arg, App, SubCommand};
use unicode_segmentation::UnicodeSegmentation;

use input::Input;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    env_logger::init().unwrap();

    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let arg_matches = App::new("rbc")
        .version(VERSION)
        .author(AUTHORS)
        .about("Counts things in text")
        .arg(
            Arg::with_name("grapheme-clusters")
                .short("c")
                .long("grapheme-clusters")
                .help("Counts the grapheme clusters")
                .long_help(
                    "Counts the grapheme clusters—what you might think \
                           of as a 'character.'",
                ),
        )
        .arg(
            Arg::with_name("FILE")
                .help("Sets the input file to use")
                .multiple(true)
                .default_value("-")
                .index(1),
        )
        /*
        .subcommand(
            SubCommand::with_name("test")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg(Arg::with_name("debug").short("d").help(
                    "print debug information verbosely",
                )),
        )
        */
        .get_matches();

    debug!("matches: {:?}", arg_matches);

    // this has a default value, so it's ok to unwrap
    let file_names: Vec<_> = arg_matches.values_of("FILE").unwrap().collect();

    let mut count: u64 = 0;
    for file_name in file_names {
        let input = Input::new(file_name)?;
        let reader = BufReader::new(input);

        for line in reader.lines() {
            let line = line?;
            count += line.graphemes(true).count() as u64;
        }
    }

    println!("grapheme clusters: {}", count);

    Ok(())
}
