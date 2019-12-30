{
  testFailed = {
    expr = builtins.add 1 1;
    expected = 1;
  };
  testPassed = {
    expr = builtins.add 1 1;
    expected = 2;
  };
  testFailed2 = {
    expr = {
      a = 1;
      b = 2;
    };
    expected = {
      a = 1;
      b = 1;
    };
  };
  testFailed3 = {
    expr = [ 1 2 3 ];
    expected = [ 1 2 4 5 ];
  };
}
