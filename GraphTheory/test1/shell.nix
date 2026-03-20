let
  pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  buildInputs = [
    
    pkgs.rustup
    
  ];

  shellHook = ''
    export PS1="(\$(basename \$(pwd))) > "
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
  '';
}
