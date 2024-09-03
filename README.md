# Stack Zero

Stack zero is a Rust based web stack that comes with a number of components and services included.

## Components and Services in use, and the Reasons Why

**Rule**: For every aspect of the stack, there is only one component or service in use. If they change, a new major version must be published. **Exception**: For short periods of time, the stack may support two to select from until consent is reached.

This list is roughly ordered by the technologies that are most robust and won't likely be changed to the ones the contributors are very unsure of and mightly likely change.

**Axum** WebServer

Axum is a very thin layer on top of hyper (the standard HTTP library in the Rust ecosystem), and provides very convenient and versatily routing abstractions and also includes the tower middleware ecosystem. It seems to scale from simple API services to large websites.

**Tera** for Templates

The Rust template engine. Inspired by Jinja2/Django and also used in Loco and Zola, the first choice.

**PostgreSQl** as the Database

I see PostgreSQL as a grown MySQL / MariaDB. Full ACID compliance and suited for scenarios with more complex queries and concurrency requirements.

**Redis** for Session Storage (Development: in-memory)

The standard for distributed session storage.

**SeaORM** as the ORM mapper

For queries, I don't think that we should not write SQL queries anymore. Specifically in the Rust ecosystem where type system integration with the DB can be provided. SeaORM is a good choice, because it is based on sqlx, is async, and supports strong migration capabilities.

Formerly, this project used Diesel, but the missing native async support and some problems with the migration utility, the authors switched to SeaORM.

**Firebase** for Authentication

Even though perhaps not really complicated, there is no way you want to roll authentication by yourself.

Formely, I thought that it's best to copy what the big guys like OpenAI are doing. Auth0. But then I realized that pricing is tremendous in production environments. So Firebase it is for now.

Status: Planned, current: auth0

**Stripe** for Payments and invoice generation

Status: Planned

## License

MIT
