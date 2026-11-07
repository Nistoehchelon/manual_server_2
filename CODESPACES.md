There are some difference between a normal machine/VPS and Codespaces, although they are somewhat minimal.

First of all, do the normal update before doing anything else:

```bash
sudo apt update && sudo apt upgrade -y
```


Postgres: pretty much as normal, except sudo service postgresql commands are used to control the db server
```bash
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y
sudo apt install postgresql-18 -y

# the only real difference
sudo service postgresql start
```

Also, there is some awkwardness here in terms of setting postgresql up:
Enter root in order to be able to run the following two sql commands
```bash
sudo -i
```

Then enter the sql commands
```bash
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
cargo install cargo-edit
```

Use sqlx-cli in repo root:
```bash
# remember to adjust .env as needed, whatever form it may take, for example, adjusting log paths and so on depending on environment
cp .env.example .env
sqlx migrate run
```


Set passwords for connecting to the codespace over SSH:
```bash
sudo passwd codespace
sudo passwd 
```
Note: Connecting over SSH happens over localhost, requires setting up port 2222 forwarding in the Codespace


Regarding postgres db management (using adminer):

use the php8.3 executable
and install php8.3 pdo pgsql extension (using apt)
```bash
sudo apt install php8.3
sudo apt install php8.3-pgsql 
```

The default php (8.4) installation does not seem to have an easy way to install extensions, but all I had to do to get it working with php8.3 was run apt install. I assume I could also just use alternatives to turn php8.3 into just php or whatever, not that it matters.


Nginx setup is as normal, except, just as postgres, sudo service nginx start/restart/stop instead of systemctl:
The main part taken from https://nginx.org/en/linux_packages.html#Ubuntu

```bash
sudo apt install curl gnupg2 ca-certificates lsb-release ubuntu-keyring -y

curl https://nginx.org/keys/nginx_signing.key | gpg --dearmor \
    | sudo tee /usr/share/keyrings/nginx-archive-keyring.gpg >/dev/null

gpg --dry-run --quiet --no-keyring --import --import-options import-show /usr/share/keyrings/nginx-archive-keyring.gpg

echo "deb [signed-by=/usr/share/keyrings/nginx-archive-keyring.gpg] \
https://nginx.org/packages/ubuntu `lsb_release -cs` nginx" \
    | sudo tee /etc/apt/sources.list.d/nginx.list

echo -e "Package: *\nPin: origin nginx.org\nPin: release o=nginx\nPin-Priority: 900\n" \
    | sudo tee /etc/apt/preferences.d/99nginx

sudo apt update
sudo apt install nginx

# the actual difference
sudo service nginx start

```

