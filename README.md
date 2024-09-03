# Stack Zero

Stack zero is a Rust based web stack that comes with  included


## Technologies in use, and the reasons why

**Rule**: For every aspect of the stack, there is only one technolgy in use. If they change, a new major version must be published. **Exception**: For short periods of time, the stack may support two to select from until consent is reached.

This list is roughly ordered by the technologies on top that are most robust and won't likely be changed.

**Axum** WebServer

Axum is a very thin layer on top of hyper (the standard HTTP library in the Rust ecosystem), and provides very convenient and versatily routing abstractions and also includes the tower middleware ecosystem. It seems to scale from simple API services to large websites.

**Tera** for Templates

The Rust template engine. Inspired by Jinja2/Django and also used in Loco and Zola, the first choice.

**PostgreSQl** as the Database

I see PostgreSQL as a grown MySQL / MariaDB. Full ACID compliance and perfectly suited for scenarios with more complex queries and concurrency requirements.

**Redis** for Session Storage (Development: in-memory)

The standard for distributed session storage.

**Firebase** for Authentication

Even though perhaps not really complicated, there is no way you want to roll authentication by yourself.

Formely, I thought that it's best to copy what the big guys like OpenAI are doing. Auth0. But then I realized that pricing is tremendous in production environments. So Firebase it is for now.

## License

MIT
