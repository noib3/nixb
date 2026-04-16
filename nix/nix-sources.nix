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
  "2_35" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "152e2880281bbbd9fa03b6215d874eec0d0dc321";
    sha256 = "0mnnyv616n1b6vf0yaqxj68zgmi8lz3m33as62qqh5dp227k17l1";
  };
}
