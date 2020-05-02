nix-test-runner
===============

Simple cli tool to run nix expression tests.

## Usage

```nix
## my-tests.nix
{
  testFailed = {
    expr = builtins.add 1 1;
    expected = 1;
  };
  testPassed = {
    expr = builtins.add 1 1;
    expected = 2;
  };
}

```

```bash
$ nix-test my-tests.nix

   ✗ testFailed

        2
        ╷
        │ Expect.equal
        ╵
        1


    TEST RUN FAILED

    Duration: 72 ms
    Passed:   1
    Failed:   1
                                    %
~

```

You can get different output formats using `--reporter junit|json|human`.

This uses `lib.debug.runTests` under the hood and doesn't change any behaviour of nix, it's merly a wrapper around `nix-instantiate`.

## From nix

Import this project into your nix build:

```bash
niv add stoeffel/nix-test-runner
```

Then in your `tests.nix` file:

```nix
let sources = import ./nix/sources.nix;
    nixTestRunner = sources.nix-test-runner;
in
  failing = nixTestRunner.runTests { testFile = ./failing.nix; };
  passing = nixTestRunner.runTests { testFile = ./passing.nix; };
  passingAlwaysPretty = nixTestRunner.runTests {
    testFile = ./passing.nix;
    alwaysPretty = true;
  };

  runninTestsFromExpression =
    nixTestRunner.runTests {
      name = "tests-from-expression";
      tests = {
        testFailed = {
          expr = builtins.add 1 1;
          expected = 1;
        };
        testPassedName = {
          expr = builtins.add 1 1;
          expected = 2;
        };
      };
    };
```

Execute all tests with e.g. `nix-build -k ./tests.nix` or include them in your
build.

For more, see inlined documentation in [default.nix](./default.nix) for
`runTests`.