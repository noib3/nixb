{ fetchFromGitHub }:

{
  "2_33" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "2.33.4";
    sha256 = "0c6ik4rcww1r135gfn324cl3ic7lly6iyz52j42jdgb2gzp7mi29";
  };
  "2_34" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "2.34.6";
    sha256 = "139j1iwlhy0yv2lr7h96d6vrrb9gq3dp007cgzvx2bp1xj334wwh";
  };
  # Current HEAD of https://github.com/NixOS/nix/pull/15675
  "2_35" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "9d718847a1b97cc8476cb23c207cacf91ba136ed";
    sha256 = "11cnvlsfv1wimhljdb5483kfd3gg3vijixf38lzssh4l3c3h7z2l";
  };
}
