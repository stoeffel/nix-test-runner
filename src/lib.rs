use clap::arg_enum;
use colored::*;
use diff;
use failure::{bail, Error};
use failure_derive::Fail;
use itertools::*;
use junit_report::{Duration, Report, TestCase, TestSuite};
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;
use std::time;
use termize;

const PASSED: &str = "TEST RUN PASSED";
const FAILED: &str = "TEST RUN FAILED";
const ARROW_DOWN: char = '▼';
const ARROW_UP: char = '▲';

#[derive(Debug, Fail)]
enum NixTestError {
    #[fail(display = "There was a problem running your nix tests.\n\n    {}", msg)]
    Running { msg: String },
}

/// Evaluates a nix file containing test expressions.
/// This uses `nix-instantiate --eval --strict` underthehood.
pub fn run(test_file: PathBuf) -> Result<TestResult, Error> {
    let run_test_nix = include_str!("./runTest.nix");
    let out = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "nix-instantiate \
             --json --eval --strict \
             -E '{run_test_nix}' \
             --arg testFile {test_file:#?}",
            test_file = test_file.canonicalize()?,
            run_test_nix = run_test_nix
        ))
        .output()
        .map_err(|msg| NixTestError::Running {
            msg: msg.to_string(),
        })?;
    if out.status.success() {
        Ok(json::from_str(from_utf8(&out.stdout)?)?)
    } else {
        bail!(NixTestError::Running {
            msg: from_utf8(&out.stderr)?.to_string()
        })
    }
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    /// Reporter used to `format` the output of `run`ning the tests.
    pub enum Reporter {
        Human,
        Json,
        Junit
    }
}

/// TestResult of running tests. Contains a field for all passed and failed tests.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    passed: Vec<PassedTest>,
    failed: Vec<FailedTest>,
}

impl TestResult {
    /// Check if all tests were successful.
    pub fn successful(&self) -> bool {
        self.failed.is_empty()
    }

    /// Format the test result given a reporter.
    pub fn format(&self, now: time::Duration, reporter: Reporter) -> Result<String, Error> {
        match reporter {
            Reporter::Json => Ok(self.json()),
            Reporter::Human => self.human(now),
            Reporter::Junit => self.junit(),
        }
    }

    fn json(&self) -> String {
        json::to_string(&self).unwrap()
    }

    fn human(&self, now: time::Duration) -> Result<String, Error> {
        Ok(format!(
            "
    {failed_tests}
    {status}

    {durationLabel} {duration} ms
    {passedLabel}   {passed_count} 
    {failedLabel}   {failed_count}

",
            status = self.status().underline(),
            durationLabel = "Duration:".dimmed(),
            passedLabel = "Passed:".dimmed(),
            failedLabel = "Failed:".dimmed(),
            duration = now.as_millis(),
            passed_count = self.passed.len(),
            failed_count = self.failed.len(),
            failed_tests = self.failed_to_human()?
        ))
    }

    fn failed_to_human(&self) -> Result<String, Error> {
        let mut failed_tests = String::new();
        for test in &self.failed {
            failed_tests.push_str(&test.human()?);
            failed_tests.push('\n');
        }
        Ok(failed_tests)
    }

    fn junit(&self) -> Result<String, Error> {
        let mut report = Report::new();
        let mut test_suite = TestSuite::new("nix tests"); // TODO use file name and allow multiple files?
        test_suite.add_testcases(self.to_testcases());
        report.add_testsuite(test_suite);
        let mut out: Vec<u8> = Vec::new();
        report.write_xml(&mut out)?;
        Ok(from_utf8(&out)?.to_string())
    }

    fn to_testcases(&self) -> Vec<TestCase> {
        let mut testcases = vec![];
        for test in &self.passed {
            testcases.push(test.junit());
        }
        for test in &self.failed {
            testcases.push(test.junit());
        }
        testcases
    }

    fn status(&self) -> ColoredString {
        if self.successful() {
            PASSED.green()
        } else {
            FAILED.red()
        }
    }
}

#[test]
fn status_passed_test() {
    assert_eq!(
        TestResult {
            passed: vec![],
            failed: vec![]
        }
        .status(),
        PASSED.green()
    )
}

#[test]
fn status_failed_test() {
    assert_eq!(
        TestResult {
            passed: vec![],
            failed: vec![FailedTest {
                expected: "".to_string(),
                result: "".to_string(),
                failed_test: "".to_string()
            }]
        }
        .status(),
        FAILED.red()
    )
}

trait JunitTest {
    fn junit(&self) -> TestCase;
    fn format_result(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FailedTest {
    expected: String,
    failed_test: String,
    result: String,
}

impl FailedTest {
    fn human(&self) -> Result<String, Error> {
        let result_diff = render_diff(ARROW_UP, &self.result, &self.expected);
        let expected_diff = render_diff(ARROW_DOWN, &self.expected, &self.result);
        let indent = 8;
        Ok(format!(
            "
    {name}

        {result}
        ╷
        │ Expect.equal
        ╵
        {expected}
        ",
            name = ("✗ ".to_owned() + &self.failed_test).red(),
            result = with_diff(&self.result, &result_diff, indent),
            expected = with_diff(&expected_diff, &self.expected, indent)
        )
        .to_string())
    }
}

fn render_diff(symbol: char, left: &str, right: &str) -> String {
    let mut rendered = String::new();
    for diff in diff::chars(left, right) {
        match diff {
            diff::Result::Left(_) => rendered.push(symbol),
            diff::Result::Right(_) => (),
            diff::Result::Both(_, _) => rendered.push(' '),
        }
    }
    rendered
}

fn with_diff(first: &str, second: &str, indent: usize) -> String {
    let width = termize::dimensions().map(|t| t.1).unwrap_or(80);
    sub_strings(first, width - indent)
        .into_iter()
        .interleave(sub_strings(second, width - indent))
        .join(&format!("\n{}", " ".repeat(indent)))
}

fn sub_strings(source: &str, sub_size: usize) -> Vec<String> {
    source
        .chars()
        .chunks(sub_size)
        .into_iter()
        .map(Iterator::collect)
        .collect::<Vec<_>>()
}

impl JunitTest for FailedTest {
    fn junit(&self) -> TestCase {
        TestCase::failure(
            &self.failed_test,
            Duration::zero(),
            "Equals",
            &self.format_result(),
        )
    }
    fn format_result(&self) -> String {
        format!(
            "Actual: {result} Expected: {expected}",
            result = self.result,
            expected = self.expected
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassedTest {
    passed_test: String,
}

impl JunitTest for PassedTest {
    fn junit(&self) -> TestCase {
        TestCase::success(&self.passed_test, Duration::zero())
    }
    fn format_result(&self) -> String {
        String::new()
    }
}
