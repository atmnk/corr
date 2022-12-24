cargo build --release
<<<<<<< Updated upstream
rm -rf /opt/homebrew/Cellar/corr/1.1.4/bin/corr
./target/release/corr --help
sudo cp ./target/release/corr /opt/homebrew/Cellar/corr/1.1.4/bin
=======
rm -rf /opt/homebrew/Cellar/corr/1.2.0/bin/corr
./target/release/corr --help
sudo cp ./target/release/corr /opt/homebrew/Cellar/corr/1.2.0/bin
>>>>>>> Stashed changes
