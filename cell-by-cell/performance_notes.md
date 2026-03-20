# Methods
Benchmarking periods 1-10 no shifts, no renders, only print to console

Mostly to check compiler optimizations


# Notes

`perf record -g cargo r --release`
`perf report`




# Results

No Optimization:
    - DNF

Release (o3):
    0.845s
    11.716s

Native CPU:
    0.900s
    ???


RUSTFLAGS="-C opt-level=3 -C target-cpu=native -Z tune-cpu=native -C target-feature=+avx2,+fma,+avx,+popcnt" cargo b --release; sleep 1; perf record -g cargo r --release


--enable-approx-func-fp-math
--enable-load-pre
--enable-loop-simplifycfg-term-folding
--enable-no-infs-fp-math
--enable-no-nans-fp-math
--enable-no-signed-zeros-fp-math
--enable-split-backedge-in-load-pre
--enable-unsafe-fp-math
--expands-variadics-override=optimize
--force-tail-folding-style=data-and-control
--fp-contract=fast
--instcombine-code-sinking
--sve-tail-folding=all
--tail-prediction=enabled
--tailcallopt
