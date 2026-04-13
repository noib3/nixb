{ fetchFromGitHub }:

{
  "2_32" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "2.32.6";
    sha256 = "1rp59n297mh6h5sbqbz5kg43j3arph5dph7yx0iwy1jzkb3gg8g5";
  };
  "2_33" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "2.33.3";
    sha256 = "0jm8wdg6iprhpja35v80cwi17nwdbklf71caq75y7d2rxzhimj6q";
  };
  "2_34" = fetchFromGitHub {
    owner = "NixOS";
    repo = "nix";
    rev = "2.34.2";
    sha256 = "0i235h58b6sncd9p7sd1f4npmccwn4jznw2rx2kflz1fq62ahqbz";
  };
}
