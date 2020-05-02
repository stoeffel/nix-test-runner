{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs {}
, sources ? import ./nix/sources.nix
, crate2nix ? sources.crate2nix
, crate2nixTools ? pkgs.callPackage "${crate2nix}/tools.nix" {}
}:

{
  package = let
      cargoNix = crate2nixTools.appliedCargoNix rec {
        name = "nix-test-runner";
        src = ./.;
      };
    in
      cargoNix.rootCrate.build;
}