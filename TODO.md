Most important one currently:

- [ ] Make sure I only set secure cookies. Currently I ask if using_https is set in env. If yes, add secure to the instruction, if no, do not. Several possibilities: 1. Just remember to set using_https to true when downstream/public facing entrypoint uses https. 2. Always set secure cookie/assume https. 3. Accept and inspect X-Forwarded-Proto and base the secure decision on that.
 



- [ ] Email verification
    Partially done: I am generating otp codes and accepting them, what remains is writing some sort of email sender and interface with external mail sending services or something.



- [ ] In general, to make the experience/functionality (I mean, UX) more comfortable, reduce friction.
     - [ ] Maybe proxy the custom site pages management like edit and view history so they appear to have their own pages for convenience.


- [ ] Rework abac for forums a bit:
I can say: subject: user, subject_id: userid123, object: forum, object_id: 33b23
so any user will be able to access forum with id 33b23
That forum might have a parent forum. Users will not have access to it, they won't even see it exists
Generally, the permissions system can be improved...



- [ ] Improve logging/tracing

- [ ] Refactor code

- [x] Remove all tls/ssl stuff from the Rust server since I am generally going to go for using nginx as reverse proxy for this, so the tls stuff only adds clutter to the Rust code.
  - [x] (I think done, not 100% sure) Remove unused tls crates?

~~- [ ] Use Nginx for serving static files and stop serving static files in the Rust App~~
    Nah, I added back the functionality to serve static in the app. Reason: The hosting provider I am using is not a full vps and does not allow starting arbitrary services, so I can only start one thing (rust app). Consider adding a feature to the rust app in order to avoid compiling static serving code when using a vps where I run nginx normally? Not that it matters too much, even if I don't play around with features, the static code path will just not be used if I capture it in nginx.
  

- [x] Change request_data.method from String into an enum, because it being a string is ridiculously brittle

- [ ] Change session flow: make sessions only contain session id, and make a table session_data with session id and key and value.


- [ ] Implement simple (and more complex, too) password bruteforcing protections.

- [x] Use git-version crate. Could be cool
- [x] Show the VERSION const on some hidden page on the live site


- [x] Log "Starting server..." on server startup. I saw restart logs and the shutdown and starting kinda blend together, so adding the starting indicator will be visually helpful.

