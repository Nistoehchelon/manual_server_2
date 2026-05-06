

- [ ] Email verification


- [ ] In general, to make the experience/functionality more comfortable, reduce friction.

- [ ] Rework abac for forums a bit:
I can say: subject user subject_id any object forum object_id 33b23
so any user will be able to access forum with id 33b23
That forum might have a parent forum. Users will not have access to it, they won't even see it exists
Generally, the permissions system can be improved...


- [ ] Maybe proxy the custom site pages management like edit and view history so they appear to have their own pages for convenience.

- [ ] Improve logging/tracing

- [ ] Refactor code

- [x] Remove all tls/ssl stuff from the Rust server since I am generally going to go for using nginx as reverse proxy for this, so the tls stuff only adds clutter to the Rust code.
  - [x] (I think done, not 100% sure) Remove unused tls crates?

- [x] Use Nginx for serving static files and stop serving static files in the Rust App

- [x] Change request_data.method from String into an enum, because it being a string is ridiculously brittle

- [ ] Change session flow: make sessions only contain session id, and make a table session_data with session id and key and value.


- [ ] Implement simple (and more complex, too) password bruteforcing protections.

- [x] Use git-version crate. Could be cool
- [x] Show the VERSION const on some hidden page on the live site


- [x] Log "Starting server..." on server startup. I saw restart logs and the shutdown and starting kinda blend together, so adding the starting indicator will be visually helpful.

