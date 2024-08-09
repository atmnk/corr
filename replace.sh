cargo build --release
rm -rf /opt/homebrew/Cellar/corr/1.2.1/bin/corr
./target/release/corr --help
sudo cp ./target/release/corr /opt/homebrew/Cellar/corr/1.2.1/bin