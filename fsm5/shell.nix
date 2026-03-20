let
  pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  buildInputs = [
    
    pkgs.rustup
    pkgs.imagemagick
    pkgs.ffmpeg
    pkgs.python3
    pkgs.python3Packages.matplotlib
    
  ];
}
