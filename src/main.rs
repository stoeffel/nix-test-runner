extern crate clap;

use clap::{arg_enum, value_t, App, Arg};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use std::time::Instant;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Reporter {
        Human,
        Json,
        Junit
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestResult {
    passed: Vec<PassedTest>,
    failed: Vec<FailedTest>,
}

impl TestResult {
    fn format(&self, now: Instant, reporter: Reporter) -> String {
        match reporter {
            Reporter::Json => serde_json::to_string(&self).unwrap(),
            Reporter::Human => format!(
                "
{failed_tests}
{status}

{durationLabel} {duration} ms
{passedLabel}   {passed_count} 
{failedLabel}   {failed_count} 
                                ",
                status = self.status(),
                durationLabel = "Duration:".dimmed(),
                passedLabel = "Passed:".dimmed(),
                failedLabel = "Failed:".dimmed(),
                duration = now.elapsed().as_millis(),
                passed_count = self.passed.len(),
                failed_count = self.failed.len(),
                failed_tests = self
                    .failed
                    .iter()
                    .fold(String::new(), |acc, t| acc + &t.human() + "\n")
            )
            .to_string(),
            Reporter::Junit => "TODO Junit".to_string(),
        }
    }

    fn status(&self) -> ColoredString {
        if self.failed.is_empty() {
            "TEST RUN PASSED".green().underline()
        } else {
            "TEST RUN FAILED".red().underline()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FailedTest {
    expected: String,
    failed_test: String,
    result: String,
}

impl FailedTest {
    fn human(&self) -> String {
        format!(
            "
{name}

    {result}
    ╷
    │ Expect.equal
    ╵
    {expected}
    ",
            name = ("✗ ".to_owned() + &self.failed_test).red(),
            result = self.result,
            expected = self.expected
        )
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassedTest {
    passed_test: String,
}

fn main() {
    let now = Instant::now();
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
    assert!(out.status.success(), "Running tests failed.");
    let res: TestResult = serde_json::from_str(from_utf8(&out.stdout).unwrap()).unwrap();
    io::stdout()
        .write_all(res.format(now, reporter).as_bytes())
        .unwrap();
}
