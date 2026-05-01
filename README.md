This document has the following parts:
1 -> Issues I would like to solve
2 -> How I usually set up the program

Issues I would like to solve
See file ISSUES.md

How I usually set this up:

2. Install PostGres:
On Windows I download zip and copy/unpack it where I want.
On Linux (say, on VPS) I followed the typical PostGres set up from the website and then some commands to create the typical db name and user with password (I usually name them database123, user123, password123).
I guess for convenience I will just put the commands here? But they might change... Maybe it is fine to just Google the postgres site like I usually do...
Note: There are discrepancies between linux vps and windows steps.
Next time I setup it on linux, I will update the steps... Because windows steps are more up to date in   terms of icu set up
On Linux:

From https://www.postgresql.org/download/linux/ubuntu/:

1.1. 
```bash
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y

sudo apt install postgresql-18 -y
sudo -u postgres psql -c "CREATE USER user123 WITH PASSWORD 'password123';"
sudo -u postgres psql -c "CREATE DATABASE database123 OWNER user123;"
cd ~/manual_server_2/pgsql_backups
chmod 600 .pgpass
# import schema into the newly installed pgsql db
echo "yes" | bash push.sh --schema
cd ..
# set up env as you wish
cp .env.example .env
```
Remember to press Enter when prompted (todo: automate the pointless interactions somehow)

1.4. test it works by: 
```bash
psql -U user123 -d database123 -h localhost -W
```
(will ask for password123)


How I created DB on Windows:

```bash
initdb --encoding=UTF8 --locale-provider=icu --icu-locale=en --data-checksums -D pgdb/
```
This has full-text search warning about und-x-icu not having fts something so it sets it to simple  mode. Available mode are simple, english, french, etc. I guess it does not matter?

```bash
createuser --superuser --pwprompt user123
```
Enter password123


```bash
createdb database123 --owner=user123 --template=template0 --encoding=UTF8
```


cd into manual_server/pgsql_backups
and use below, without the chmod 600 a warning that prevents script execution occurs:
WARNING: password file "/home/username/manual_server/pgsql_backups/.pgpass" has group or world access; permissions should be u=rw (0600) or less



```bash
cargo run
```


1. Ah, and the local (for dev) self-signed cert generation/TLS intrusctions is inside local_ssl/README.md
2. This seems to be about it



Regarding pgsql_backups:
I have two scripts there: push.sh and pull.sh
When I want to backup contents of db I use (--full backups data and --schema only the empty tables. Both can be used, they are stored separately):
```bash
bash pull.sh --full
```
```bash
bash pull.sh --schema
```

And when I want to import I do (again, --full or --schema depending on what I want):
```bash
bash push.sh --full
```
```bash
bash push.sh --schema
```

