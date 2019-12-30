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
