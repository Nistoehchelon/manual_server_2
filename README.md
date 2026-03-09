This document has the following parts:
1 -> Issues I would like to solve
2 -> How I usually set up the program

Issues I would like to solve
See file ISSUES.md

How I usually set this up:

0. If using VPS:
Remember to

```bash
sudo apt install build-essential
```

, required to run rust compilation.
Also, if using VSCode's Remote SSH, it seems that while it connects to a Fedora VPS works out of the box, when Remote SSH connects to Ubuntu VPS, it times out. I disabled "Use Local Server" and then it seemed to have begun working.


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
```

1.2. 
```bash
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
```

^ The above 1.1 and 1.2 pull postgres's up to date repo. Typically, a fresh Ubuntu VPS will have an outdated Postgres  like 16 (current latest is 18, at the moment I write this)
Then, of course, the latest postgres is available (replace 18 with needed version):
1.3. 
```bash
sudo apt install postgresql-18
```

Then setup db and user:
1.4. 
```bash
sudo -u postgres psql <- login as postgres user
```
1.5. 
```sql
CREATE USER user123 WITH PASSWORD 'password123';
```
1.6. 
```sql
CREATE DATABASE database123 OWNER user123;
```
1.7. GPT suggested this, but probably not necessary: 
```sql
ALTER ROLE user123 WITH LOGIN;
```
1.8. test it works by: 
```bash
psql -U user123 -d database123 -h localhost -W
```
(will ask for password123)


How I created DB on Windows:

```bash
initdb --encoding=UTF8 --locale-provider=icu --icu-locale=und-x-icu  --lc-collate='und-x-icu' --lc-ctype='und-x-icu' --lc-messages='und-x-icu' --lc-monetary='und-x-icu' --lc-numeric='und-x-icu' --lc-time='und-x-icu' --data-checksums -D pgdb/
```
This has full-text search warning about und-x-icu not having fts something so it sets it to simple  mode. Available mode are simple, english, french, etc. I guess it does not matter?


On Termux:

```bash
ibitdb --encoding=UTF8 --locale-provider=icu --icu-locale='und-x-icu --data-checksums -D pgdb/'
```


```bash
createuser --superuser --pwprompt user123
```
Enter password123


```bash
createdb database123 --owner=user123 --template=template0 --encoding=UTF8
```



1. Configure .env file based on .env_example. On fresh set up I can copy the example and modify as needed.
2. I guess pg_backups/.pgpass if needed, but I usually use default password123 etc for dev and pg pass is only for dev, right? I dunno, by the way, how production import/export would look like.
3. Use pg_backups's pull and push scripts...
4. Ah, and the local (for dev) self-signed cert generation/TLS intrusctions is inside local_ssl/README.md
5. This seems to be about it

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

