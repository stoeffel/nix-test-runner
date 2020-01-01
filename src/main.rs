extern crate clap;

use clap::{value_t, App, Arg};
use std::error::Error;
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
        ])
        .get_matches();
    let reporter = value_t!(matches, "reporter", nix_test_runner::Reporter).unwrap();
    let test_file_path = PathBuf::from(matches.value_of("TEST").unwrap());
    let output = matches.value_of("output").map(|o| Path::new(o));
    assert!(
        test_file_path.exists(),
        "You need to provide an existing file."
    );
    match nix_test_runner::run(test_file_path) {
        Ok(result) => {
            let formatted = result.format(now.elapsed(), reporter);
            match output {
                None => io::stdout().write_all(formatted.as_bytes()).unwrap(),
                Some(output_path) => {
                    let display = output_path.display();

                    let mut file = File::create(&output_path).unwrap_or_else(|why| {
                        panic!("Couldn't create {}: {}", display, why.description())
                    });

                    match file.write_all(formatted.as_bytes()) {
                        Err(why) => panic!("Couldn't write to {}: {}", display, why.description()),
                        Ok(_) => println!("Successfully wrote to {}", display),
                    }
                }
            }
            process::exit(if result.successful() { 0 } else { 1 })
        }
        Err(err) => {
            io::stderr().write_all(err.as_bytes()).unwrap();
            process::exit(1)
        }
    }
}
