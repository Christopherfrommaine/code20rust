let
  pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  buildInputs = [
    
    pkgs.rustup
    
  ];
  
  shellHook = ''
    export PS1='\[\e[1;31m\][\W]$ \[\e[0m\]'
    
  '';
}
