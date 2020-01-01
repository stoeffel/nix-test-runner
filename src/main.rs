extern crate clap;

use clap::{value_t, App, Arg};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

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
        ])
        .get_matches();
    let reporter = value_t!(matches, "reporter", nix_test_runner::Reporter).unwrap();
    let test_file_path = PathBuf::from(matches.value_of("TEST").unwrap());
    assert!(
        test_file_path.exists(),
        "You need to provide an existing file."
    );
    match nix_test_runner::run(test_file_path) {
        Ok(result) => {
            let formatted = result.format(now.elapsed(), reporter);
            io::stdout().write_all(formatted.as_bytes()).unwrap();
            process::exit(if result.successful() { 0 } else { 1 })
        }
        Err(err) => {
            io::stderr().write_all(err.as_bytes()).unwrap();
            process::exit(1)
        }
    }
}
