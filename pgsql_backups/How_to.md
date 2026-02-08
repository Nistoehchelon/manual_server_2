0. cd pg_sql
1. nano .pgpass
2. enter: localhost:5432:database123:user123:password123
3. chmod 600 .pgpass
4. Export Schema only: PGPASSFILE=.pgpass pg_dump -U user123 -d database123 --schema=public --schema-only --clean > public_schema.sql
5. Export Full: PGPASSFILE=.pgpass pg_dump -U user123 -d database123 --schema=public --clean > public_full.sql
6. Import schema: PGPASSFILE=.pgpass psql -h localhost -U user123 -d database123 -f public_schema.sql
7. Import full: PGPASSFILE=.pgpass psql -h localhost -U user123 -d database123 -f public_full.sql


Notes: 
1. Might be useful: 
chmod +x pull.sh push.sh
chmod 600 .pgpass

2. The push.sh and pull.sh scripts can be run to push and pull. I am not one hundred percent sure they are foolproof, though, because I did not bother inspecting them too closely after copying from deepseek.

3. The pg_dump's --clean option on pull commands allows for subsequent pushes of the resultant file to override whatever 
current state the db happens to be in at the push point by (IIRC) first clearing the db from the tables. Without --clean,
I would have to call DROP for each table or some such to avoid errors of the nature "already exists".