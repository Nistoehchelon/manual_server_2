This document has the following parts:
1 -> Issues I would like to solve
2 -> How I usually set up the program

Issues I would like to solve

There is an issue that can be observer when the following happens: Often, but not always, when I do this: login, logout, I see a 400 error (remember, the web server is written by me, so it returns this because I made it do that). 
I am not sure what causes this unclear issue.
Several possibilities:
1. Problem is in logic encoded in the code.
2. Maybe it's just PostGres acting up? In other words, I need to look into the postgres setup rather than trying to fix the app code
 
I would assume this particular issue might come from code. 
However, I do notice that postgres is acting up too, for example it no longer starts up on laptop start up unlike before and I have to do this manually, not sure why. Maybe something switched around after I manually did "pg_ctl restart" when testing ssl stuff? Since then, I had to manually start it.
From what the logs say at the point the 400 error happens, it seems the header reading fails. In other words, it seems that client is not sending anything sometimes? But it kinda should? Or not? Should I try ignoring it and sending nothing, just maybe closing connection? or waiting more?

It seems to primarily happen during login/logout. I have not tested message sending. But maybe the issue is with redirects? Maybe it is due to keep alive implementation I am playing around with? Maybe it has to do with flush? Do I need to flush before stream shutdown after breaking keep alive loop? Maybe the issue is with keeping alive after redirect?

I
What I tried:
Changing redirect to always be connection close. But did not seem like this fixed anything. Fair enough.
If I close connection at redirect, the  next connection also has this issue, seemingly.

What I have not tried:
Seeing what happens if instead of returning 400 I just continue if byte reading time outs...

What I will do now:
I have had no success in solving  this on my own. I suppose it might be time to focus on something else and deal with the random 400's for now. I suppose if nothing will pop up in the mind regarding this, I will ask for help on stackoverflow.com or something.

New thoughts:
Perhaps the issues lies in buf reader and keep alive implementation. I may consider using the previous simple stream read rather than reader read and using connection close everywhere. What will I do if the problem disappears once I do that? Well, I could go like this: Since neither keep alive nor buf reader are necessary either for security (unlike, say, tls) nor functionality, I can just avoid using them for now. The main reason why I used bufreader is because it seemed that keep alive requires it.
What if removing keep alive and buf reader does not solve the issue: I don't know what to do to solve this aside from "just work in something else for at least a while" and "ask people for help". In other words, I suppose I could say "I would be back where I started". Well, that's not quite true, is it? I gues another phrasing could go like: I would continue doing other things, maybe ask for help at some point, supposedly some new ideas might occur to this mind.



How I usually set this up:
1. Install PostGres:
On Windows I used the GUI installer.
On Linux (say, on VPS) I followed the typical PostGres set up from the website and then some commands to create the typical db name and user with password (I usually name them database123, user123, password123).
I guess for convenience I will just put the commands here? But they might change... Maybe it is fine to just Google the postgres site like I usually do...
I guess I will also put the commands here...
From https://www.postgresql.org/download/linux/ubuntu/:

1.1. sudo apt install -y postgresql-common
1.2. sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh

^ The above 1 and 2 pull postgres's up to date repo. Typically, a fresh Ubuntu VPS will have an outdated Postgres  like 16 (current latest is 18, at the moment I write this)
Then, of course, the latest postgres is available (replace 18 with needed version):
1.3. sudo apt install postgresql-18

Then setup db and user:
1.4. sudo -u postgres psql <- login as postgres user
1.5. CREATE USER user123 WITH PASSWORD 'password123';
1.6. CREATE DATABASE database123 OWNER user123;
1.7. GPT suggested this, but probably not necessary: ALTER ROLE user123 WITH LOGIN;
1.8. test it works by: psql -U user123 -d database123 -h localhost -W (will ask for password123)



2. Configure .env file
3. I guess pg_backups/.pgpass if needed, but I usually use default password123 etc for dev and pg pass is only for dev, right? I dunno, by the way, how production import/export would look like.
4. Use pg_backups's pull and push scripts...
5. Ah, and the local (for dev) self-signed cert generation inside local_ssl
6. This seems to be about it

