run:
    cargo run
    
run-release:
    cargo run --release

build:
    cargo build --release

test:
    cargo test

bench:
    cargo bench

profile:
    RUSTFLAGS="-C llvm-args=--inline-threshold=0 -C force-frame-pointers=yes" cargo bench --bench caminos -- --profile-time 1

clean:
    cargo clean
