1. 
```bash
/usr/lib/postgresql/18/bin/pg_ctl -D pgdb -o "-c unix_socket_directories=/tmp" -l logfile start
```

2.
```bash
sudo rm -f /etc/apt/sources.list.d/yarn.list
```