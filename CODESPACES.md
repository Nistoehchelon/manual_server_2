```bash
sudo apt update && sudo apt upgrade -y
```

```bash
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y
sudo apt install postgresql-18 -y
sudo service postgresql start

# enter root in order to be able to run the following two sql commands
sudo -i
sudo -u postgres psql -c "CREATE USER user123 WITH PASSWORD 'password123';"
sudo -u postgres psql -c "CREATE DATABASE database123 OWNER user123;"

# remember to exit root, because if you continue being in it beyond now, you might forget about it and this might cause annoying
# issues like the following: running cargo run from root shell and rust-analyzer from normal user causing full recompilation on every
# file save/cargo build
exit

```

Installing Rust and sqlx-cli: like normal.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
cargo install sqlx-cli
```
Use sqlx-cli in repo root:
```bash
# remember to adjust .env as needed, whatever form it may take, for example, adjusting log paths and so on depending on environment
cp .env.example .env
sqlx migrate run
```


Set passwords for connecting to SSH:
```bash
sudo passwd codespace
sudo passwd 
```

Compiling on MUSL:
Why? Well, previously, I was compiling the release executable on one machine and hosting it on another, both Ubuntu, and it worked well. However, I now tried this on another machine and it gave an error about something like "glic not found" or whatever. From what I read that true static linking is Musl but it might come with caveats. I am not sure what this will look in practice, but MUSL executable worked on both machines, so maybe it works fine, let's see. 

```bash
rustup target add x86_64-unknown-linux-musl

sudo apt install musl-tools

cargo build --release --target x86_64-unknown-linux-musl

```


Regarding postgres db management (using adminer):

use the php8.3 executable
and install php8.3 pdo pgsql extension (using apt)

The default php (8.4) installation does not seem to have an easy way to install extensions, but all I had to do to get it working with php8.3 was run apt install. I assume I could also just use alternatives to turn 8.3 into just php or whatever.

