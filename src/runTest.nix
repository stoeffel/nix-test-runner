{ testFile, lib ? (import <nixpkgs> { }).lib }:
with builtins;
let
  tests = import testFile;
  testNames = map (t: { passedTest = t; }) (attrNames tests);
  failed = map (t: {
    failedTest = t.name;
    expected = toJSON t.expected;
    result = toJSON t.result;
  }) (lib.debug.runTests tests);
  failedTests = map (f: f.failedTest) failed;
  passed = filter (t: !lib.elem t.passedTest failedTests) testNames;
  result = { inherit passed failed; };
in result
