Here I list some issues that have occurred.

There was an issue that could be observed when the following happened: Often, but not always, when I do this: login, logout, I see a 400 error (remember, the web server is written by me, so it returns this because I made it do that). 
I am not sure what caused this unclear issue.

I have solved it, below is the solution. And after the solution, I have the text I wrote before I found the solution.


Solution:
It seems that the issue was caused by browser/client opening sockets but doing nothing. I (on server) treated this as a bad request and sent 400 errors. 
I was already familiar with the idea that some client requests are to be ignored.
But I did not initially thoughts of this...
Eventually, I saw that the error seemed to occur whenever the server received no headers, not even anything.
So, I made it so if client socket timeouts AND they have not sent even a single byte, I just silently close the connection. WIthout sending any response like 400.
I have to say, I am surprised, because I thought that if browser starts an unused connection, it will ignore any response... But apparently this is not so. So, I need to silently close the connection, and silently is the keyword, otherwise the actual user of the browser will see annoying 400 errors...


Past, before solution:
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
New thoughts 2:
I noticed it seems to happen somewhat randomly. I tested by just reloading pages, not only login/logout, and it triggers every 6-7-8 reloads at times but can also trigger at every other reload, etc. I think this might be caused somehow by keep alive and buf reader. I might wanna try this: Rollback my code to NOT use keep alive, just close after every request and 2) Use the old way of reading stream, maybe even with left over bytes.
Of course, it is possible that the issue lies simply in TLS setup somehow? Could it lie in the fact that I am using local setup somehow?.. 

New thoughts 3:
After reinstalling the whole app (without modifications) I noticed none of the 400 errors yet. I don't get why.  
