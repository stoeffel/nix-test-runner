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
