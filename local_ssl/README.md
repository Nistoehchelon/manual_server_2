So, I am using this to allow connecting to https when I am on localhost (for example, developing) for both site and db.
Steps are basically: generate certs for site and for db, add them to 
(On PC Chrome: 
```text 
chrome://certificate-manager/localcerts/usercerts
```
for site and at wherever the db is located in postges.conf for postgers cert). OR to get to the cert menu manually: Chrome Settings -> Search for "certificate" -> Security -> Manage Certificates -> Custom (Installed by you) -> Trusted Certificates Import


And on Android phone: Somewhere in settings. If when file browsing you see no files in the system picker, you might wanna first enter system picker outside the context of certificate installation (might appear in recent apps or just find it somewhere in app list) and then go to a folder where the cert is located. Then return to settings -> cert installation and the picker should show this folder. At least, this happened on the phone I used. 


1.
Generate the (self-signed) cert for visiting the local website at https://127.0.0.1:{port} using the command:

 ```bash
 openssl req -x509 -nodes -days 3650 -newkey rsa:2048 -keyout localhost.key -out localhost.crt -config localhost.config
```


2. 
And this one is for postgres, to be used like this: both set the path in postgres config and in app config using database url:

```bash
openssl req -x509 -nodes -days 3650 -newkey rsa:2048 -keyout postgres.key -out postgres.crt -config postgres.config
``` 
    
The only difference between the localhost and postgres is that v3_req section for localhost has basicConstraints=CA:TRUE (Android requires it, apparently... Because without it, it fails to load the cert)
    while Postgres does not work with it.
    

2. So, that's it... The Rust app will now pull the key at localhost.key and localhost.cert and use it when in ENVIRONMENT=local mode (set in the ".env")



For Postgres:

1. Edit file C:\Program Files\PostgreSQL\18\data\postgresql.conf:
	```text
 	ssl = on
	ssl_cert_file = 'path/to/repo/local_ssl/postgres.crt'
	ssl_key_file = 'path/to/repo/local_ssl/postgres.key'
	ssl_min_protocol_version = 'TLSv1.2'
 	```

2. ```bash
   chmod 600 localhost.key
   ```

3. Edit C:\Program Files\PostgreSQL\18\data\pg_hba.conf:
```text
hostssl all 		all 		127.0.0.1/32 		scram-sha-256
hostssl all 		all 		::1/128      		scram-sha-256
```

4. Restart postgers:
```bash
 pg_ctl  -D "path/to/pgdb" stop
```
```bash
 pg_ctl  -D "path/to/pgdb" start
```

 6. Remember to edit the sslrootcert file path if needed:
```text
postgres://user:password@localhost/dbname?sslmode=verify-full&sslrootcert=path/to/repo_folder/local_ssl/localhost.crt
```
