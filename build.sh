rm -rf output
cargo init output
cargo run
cp -r internals/internals output/src/internals
cd output
cargo build
cd ..
cp target/debug/output ./a
rm -rf output