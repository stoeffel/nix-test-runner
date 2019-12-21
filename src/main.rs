extern crate clap;
use clap::{arg_enum, value_t, App, Arg};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Reporter {
        Human,
        Junit
    }
}

#[derive(Debug)]
struct Arguments<'a> {
    test_file: &'a str,
    reporter: Reporter,
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
    let arguments = Arguments {
        test_file,
        reporter,
    };
    println!("Using arguments: {}", arguments.reporter);
}
