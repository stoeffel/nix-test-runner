extern crate clap;

use clap::{value_t, App, Arg};
use failure::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

static NAME: &str = env!("CARGO_PKG_NAME");
static VERSION: &str = env!("CARGO_PKG_VERSION");
static AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
static DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let now = Instant::now();
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .args(&[
            Arg::from_usage(
                "<TEST> +required
                'A nix expression containing testcases.'",
            ),
            Arg::from_usage("-r, --reporter 'Reporter to display the test results.'")
                .default_value("Human")
                .possible_values(&nix_test_runner::Reporter::variants())
                .case_insensitive(true),
            Arg::from_usage("-o, --output=[FILE] 'Specify output file for test results.'"),
            Arg::from_usage(
                "--skip-run 'The given input file is already the test output as JSON.'",
            ),
        ])
        .get_matches();
    let reporter = value_t!(matches, "reporter", nix_test_runner::Reporter).unwrap();
    let test_file_path = PathBuf::from(matches.value_of("TEST").unwrap());
    let output = matches.value_of("output").map(|o| Path::new(o));
    let skip_run = matches.is_present("skip-run");
    assert!(
        test_file_path.exists(),
        "You need to provide an existing file."
    );

    let test_result: Result<nix_test_runner::TestResult, _> = if skip_run {
        serde_json::from_reader(File::open(test_file_path).expect("could not open file"))
            .map_err(|e| e.into())
    } else {
        nix_test_runner::run(test_file_path)
    };

    match test_result {
        Ok(result) => {
            let now_for_elapsed_reporting = if skip_run { None } else { Some(now) };
            formatting(&result, reporter, output, now_for_elapsed_reporting).unwrap();
            process::exit(if result.successful() { 0 } else { 1 })
        }
        Err(err) => {
            io::stderr().write_all(err.to_string().as_bytes()).unwrap();
            process::exit(1)
        }
    }
}

fn formatting(
    result: &nix_test_runner::TestResult,
    reporter: nix_test_runner::Reporter,
    output: Option<&Path>,
    now: Option<Instant>,
) -> Result<(), Error> {
    let formatted = result.format(now.map(|instant| instant.elapsed()), reporter)?;
    match output {
        None => io::stdout().write_all(formatted.as_bytes())?,
        Some(output_path) => {
            let display = output_path.display();
            let mut file = File::create(&output_path)?;

            file.write_all(formatted.as_bytes())?;
            println!("Successfully wrote to {}", display);
        }
    };
    Ok(())
}
