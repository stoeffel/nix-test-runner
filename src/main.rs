extern crate clap;
use clap::{arg_enum, value_t, App, Arg};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Reporter {
        Human,
        Junit
    }
}

fn main() {
    let matches = App::new("nix-test")
        .version("0.0.1")
        .author("Christoph H. <schtoeffel@gmail.com>")
        .about("Run nix expression tests.")
        .arg(
            Arg::with_name("TEST")
                .required(true)
                .index(1)
                .help("A nix expression containing testcases."),
        )
        .arg(
            Arg::with_name("reporter")
                .required(false)
                .short("r")
                .long("reporter")
                .default_value("human")
                .possible_values(&Reporter::variants())
                .case_insensitive(true),
        )
        .get_matches();
    let test_file = matches.value_of("TEST").unwrap();
    let reporter = value_t!(matches, "reporter", Reporter).unwrap_or_else(|e| e.exit());
    println!("Using : {}", test_file);
    println!("Using : {}", reporter);
    let test_file_path = Path::new(test_file);
    assert!(
        test_file_path.exists(),
        "You need to provide an existing file."
    );
    // TODO Case windows?
    let out = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "nix-instantiate \
             --json --eval --strict \
             ./runTest.nix \
             --arg testFile {}",
            test_file
        ))
        .output()
        .expect("failed to execute process");
    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
    assert!(out.status.success());
}
