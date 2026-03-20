let
  pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.imagemagick
  ];
  
  shellHook = ''
    export RUST_LOG=info
    export RUSTUP_TOOLCHAIN=nightly
    export RUSTFLAGS='-C opt-level=3 -C target-cpu=native -Z tune-cpu=native -C target-feature=+avx2,+fma,+avx,+popcnt,+sse3 -C llvm-args=-fp-contract=fast -C llvm-args=--tailcallopt -C llvm-args=--tail-predication=enabled -C llvm-args=--enable-loop-simplifycfg-term-folding -C llvm-args=--instcombine-code-sinking -C llvm-args=--sve-tail-folding=all -C llvm-args=--enable-load-pre -C llvm-args=--expand-variadics-override=optimize -C llvm-args=--instcombine-negator-enabled -C llvm-args=--iterative-counter-promotion'
  '';
}
