1. 
```bash
/usr/lib/postgresql/18/bin/pg_ctl -D pgdb -o "-c unix_socket_directories=/tmp" -l logfile start
```

2.
```bash
sudo rm -f /etc/apt/sources.list.d/yarn.list
```

3.
```bash
export PATH=$PATH:/usr/lib/postgresql/18/bin
```

The app does not work. The codespaces proxy gives 502 error. A simple python server works.
TODO:
1.Test if app works using nginx reverse proxy on a VPS.
2. Test if app works in codespaces using three layers: app -> nginx -> codespaces.
I assume something is wrong with my web server implementation.
