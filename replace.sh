cargo build --release
rm -rf /opt/homebrew/Cellar/corr/0.1.27/bin/corr
./target/release/corr --help
sudo cp ./target/release/corr /opt/homebrew/Cellar/corr/0.1.27/bin