{ sources ? import ../nix/sources.nix
, nixpkgs ? sources.nixpkgs
, pkgs ? import nixpkgs {}
}:

let nixTestRunner = pkgs.callPackage ../default.nix {};
in
{
  failing = nixTestRunner.runTests { testFile = ./failing.nix; };
  passing = nixTestRunner.runTests { testFile = ./passing.nix; };
  passingAlwaysPretty = nixTestRunner.runTests { testFile = ./passing.nix; alwaysPretty = true; };

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
}