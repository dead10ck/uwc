/// Note that because std::process::Output::std{out,err} is just a Vec<u8> and
/// OsString::from_vec is unstable, these tests assume that stdout is valid UTF-8.

use std::collections::VecDeque;
use std::process::Command;
use std::ffi::OsStr;

fn main_binary() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("-q");
    cmd
}

fn main_binary_with_args<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = main_binary();
    cmd.arg("--");
    cmd.args(args);
    cmd
}

/// Takes a String that should be the output of a run, discards the header, and
/// parses the rest of the output into their fields.
fn parse_lines<'a>(output: &'a str, has_header: bool) -> Vec<(Vec<usize>, &'a str)> {
    let mut lines: VecDeque<&str> = output.lines().collect();

    // If there's a header, there should be at least 2 lines. If there is no
    // header, there should be at least one.
    let min_lines = if has_header { 2 } else { 1 };

    assert!(lines.len() >= min_lines, "bad output: {}", output);

    // discard the header if it has one
    if has_header {
        lines.pop_front();
    }

    let mut parsed = Vec::new();

    for line in lines {
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        let fname = fields.pop().unwrap();
        parsed.push((
            fields
                .into_iter()
                .map(str::parse)
                .map(Result::unwrap)
                .collect(),
            fname,
        ));
    }

    parsed
}

/// Tests that the CLI run with no arguments prints the header with the default
/// counters and all 0s.
#[test]
fn test_no_args() {
    let out = main_binary().output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let fields = parse_lines(&stdout, true);

    // lines  words  bytes  filename
    // 0      0      0      -
    let correct_fields = vec![(vec![0usize, 0, 0], "-")];
    assert_eq!(correct_fields, fields);

    // should be no stderr
    let stderr = out.stderr;
    assert_eq!(0, stderr.len());
}

/// Tests that the CLI run with no arguments prints the header with the default
/// counters and all 0s.
#[test]
fn test_no_args_no_elastic_tabs() {
    let out = main_binary_with_args(&["--no-elastic"]).output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let correct_output = String::from(
        r#"lines	words	bytes	filename
0	0	0	-
"#,
    );

    assert_eq!(correct_output, stdout);

    // should be no stderr
    let stderr = out.stderr;
    assert_eq!(0, stderr.len());
}

