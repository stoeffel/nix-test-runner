nix-test-runner
===============

Simple cli tool to run nix expression tests.

## Usage

```nix
## my-tests.nix
{
  testPassedName = {
    expr = add 1 1;
    expected = 2;
  };
}
```

```bash
$ nix-test my-tests.nix
```

You can get different output formats using `--reporter junit|json|human`.

This uses `lib.debug.runTests` under the hood and doesn't change any behaviour of nix, it's merly a wrapper around `nix-instantiate`.
