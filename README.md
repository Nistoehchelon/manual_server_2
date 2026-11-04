# Postgres
  Before compilation, ensure that the state of the Postgres DB abides by the following:
  1. First of all, Postgres has to be installed and running,
  2. And it should have a database called database123 and a user called user123 with password password123 (if you wanna change the default names and password, feel free to edit the .env file and if you are gonna use the pgsql_backups scripts, don't forget to edit .pgpass there, too to reflect the new names and password)
  3. Finally, you should have applied the sqlx migrations

The reason for this is that I am using the sqlx mode that connects to the db at compile time (for schema checking).
Once all the Postgres conditions are met, you may proceed to the compilation steps, outlined in ABOUT_COMPILATION.md.  


Sqlx migrations:

```bash
cargo install sqlx-cli
sqlx migrate run
```


# If I want to extract/import db contents with data.

I have two scripts in pgsql_backups: push.sh and pull.sh, I may use them for exporting and importing DB contents (this can also be done in any other way, like, say, exporting/importing through phpmyadmin or adminer). In the past, I used the --schema option, too, but this use case was replaced by me using the sqlx's migration thingie.

```bash
bash pull.sh --full
```
```bash
bash pull.sh --schema
```

And when I want to import I could do:
```bash
bash push.sh --full
```
```bash
bash push.sh --schema
```


