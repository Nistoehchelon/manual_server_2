1.
In general, to make the experience/functionality more comfortable, reduce friction.

2. Rework abac for forums a bit:
I can say: subject user subject_id any object forum object_id 33b23
so any user will be able to access forum with id 33b23
That forum might have a parent forum. Users will not have access to it, they won't even see it exists
Generally, the permissions system can be improved...


3. Maybe proxy the custom site pages management like edit and view history so they appear to have their own pages for convenience.

4. Improve logging/tracing

5. Refactor code

6. Remove all tls/ssl stuff from the Rust server since I am generally going to go for using nginx as reverse proxy for this, so the tls stuff only adds clutter to the Rust code.

7. Use Nginx for serving static files and stop serving static files in the Rust App

8. Change request_data.method from String into an enum, because it being a string is ridiculously brittle