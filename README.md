# generate-long-duration-jwt

### Building locally
- Install rust
- install python3
- create virtualenvironment & setup cargo zig
```shell
python3 -m venv .venv
source .venv/bin/activate
pip3 install -U pip
pip3 install -r requirements.txt
cargo install --locked cargo-zigbuild
rustup target add x86_64-unknown-linux-musl
cargo zigbuild --target x86_64-unknown-linux-musl --release
```

### tests
- Simply run `cargo test --all-features`
