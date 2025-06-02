# used for termux, installs a new version of this backend
rm -r .bootstrap
mkdir .bootstrap
cd .bootstrap
git clone https://github.com/no-venv/guestboard-backend-2
cd guestboard-backend-2
cargo build --release
cp target/release/backend ~/backend
