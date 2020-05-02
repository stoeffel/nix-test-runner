{ sources ? import ./nix/sources.nix
, nixpkgs ? sources.nixpkgs
, pkgs ? import nixpkgs {}
, crate2nix ? sources.crate2nix
, crate2nixTools ? pkgs.callPackage "${crate2nix}/tools.nix" {}
}:

rec {
  package = let
      cargoNix = crate2nixTools.appliedCargoNix rec {
        name = "nix-test-runner";
        src = ./.;
      };
    in
      cargoNix.rootCrate.build;

  /* Runs your nix tests from a file or an expression
     and outputs a pretty diff if they fail.

     Type: runTests attrSet -> derivation

     Example:
      runTests { testFile = ./examples/failing.nix; }
      => returns a derivation that will show a failure diff.
      runTests {
        tests = {
          testFailed = {
            expr = builtins.add 1 1;
            expected = 1;
          };
        };
      }
      => returns a derivation that will show a failure diff.

    You need to pass one of the following arguments:

      testFile - the nix file to import that evaluates to the nix expressions.
      tests    - the nix expression containing the tests. Takes precedence.

    Optional arguments:

      name         - used in the derivation(s) produced (for the test results as
                     JSON etc.).
      alwaysPretty - also print pretty results for passing tests.

    If there are no failures, returns a derivation with an empty output.
   */
  runTests =
    { name ?
        if testFile != null
        then "nix-tests-${builtins.baseNameOf testFile}"
        else "nix-tests"
    , testFile ? null
    , tests ? import testFile
    , alwaysPretty ? false
    }:
    let result = testResult { inherit tests; };
        debugTestOrigin =
          if testFile != null
          then "${name} imported from ${toString testFile}"
          else name;
        resultJson = pkgs.writeTextFile {
          name = "${name}-result.json";
          text = builtins.toJSON result;
        };
        failed = result.failed or [];
        allGood = failed == [];
    in
      if allGood
      then (
        if alwaysPretty
        then pkgs.runCommandLocal "${name}-passed" {}
          ''
          echo -e "\e[1mPASSED\e[0m: ${debugTestOrigin}"
          touch $out
          ''
        else pkgs.runCommandLocal "${name}-passed" {}
          ''
          echo -e "\e[1mPASSED\e[0m: ${debugTestOrigin}"
          echo ""
          (
            set -x
            ${package}/bin/nix-test --skip-run ${resultJson} | tee $out
          )
          ''
      )
      else pkgs.runCommandLocal
        "${name}-failed"
        {}
        ''
        echo -e "\e[1m\e[31mFAILED\e[0m: ${debugTestOrigin}"
        echo ""
        (
          set -x
          ${package}/bin/nix-test --skip-run ${resultJson}
        )
        '';

  /* Returns the prettified test results as processed by nix-test-runner. */
  testResult = import ./src/runTest.nix;
}