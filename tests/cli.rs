/// Note that because std::process::Output::std{out,err} is just a Vec<u8> and
/// OsString::from_vec is unstable, these tests assume that stdout is valid UTF-8.

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

/// Tests that the CLI run with no arguments prints the header with the default
/// counters and all 0s.
#[test]
fn test_no_args() {
    let out = main_binary().output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
//     let correct_output = String::from(
//         r#"lines	words	bytes	filename
// 0	0	0	-
// "#,
//     );

    assert_eq!(correct_output, stdout);

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
