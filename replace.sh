cargo build --release
sudo rm /opt/homebrew/Cellar/corr/0.1.25/bin/corr
sudo cp ./target/release/corr /opt/homebrew/Cellar/corr/0.1.25/bin