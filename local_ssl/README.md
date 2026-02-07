1.
Generate the (self-signed) cert using the command:
    openssl req -x509 -nodes -days 3650 -newkey rsa:2048 -keyout localhost.key -out localhost.crt -config localhost.config

    openssl req -x509 -nodes -days 3650 -newkey rsa:2048 -keyout postgres.key -out postgres.crt -config postgres.config 
    
    the only difference between the localhost and postgres is that v3_req section for localhost has basicConstraints=CA:TRUE (Android requires it, apparently...)
    while Postgres does not work with it.
    

2. So, that's it... The Rust app will now pull the key and use it when in developing mode (set in the "config" file in root)

3. For convenience (i.e., to avoid having to constantly click "skip warning" about untrusted cert in, say, Chrome), the localhost.crt may be added into, say, Chrome's custom certificate store (Chrome Settings -> Search for "certificate" -> Security -> Manage Certificates -> Custom (Installed by you) -> Trusted Certificates Import)

For Postgres:

1. Edit file C:\Program Files\PostgreSQL\18\data\postgresql.conf:
ssl = on
ssl_cert_file = 'C:/Users/User/rust/manual_server_with_workspaces/local_ssl/localhost.crt'
ssl_key_file = 'C:/Users/User/rust/manual_server_with_workspaces/local_ssl/localhost.key'
ssl_min_protocol_version = 'TLSv1.2'

2. chmod 600 localhost.key

3. Edit C:\Program Files\PostgreSQL\18\data\pg_hba.conf:
hostssl all all 127.0.0.1/32 scram-sha-256
hostssl all all ::1/128      scram-sha-256

4. Restart postgers:
 pg_ctl restart -D "C:\Program Files\PostgreSQL\18\data"

 5. Remember to edit the sslrootcert file path if needed: postgres://user:password@localhost/dbname?sslmode=verify-full&sslrootcert=localhost.crt
