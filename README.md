# Reduce

Reduce is a productivity tool that focuses on simplicity and intent. To achieve this, several
specific types of information are handled:

- Time usage
- Tasks
- Information to process
- Documents

To understand how to use Reduce, you should understand its 2 primary goals. The first of which is
simplicity. Simplicity really entails 2 things. One of which is simplicity in technology. Reduce is
built using a server-first architecture, using HTMX sparingly where appropriate. This means the
resulting system is much more robust, reducing bugs, security issues, and maintainment cost. Another
way reduce is simple is in how you use it. Data has limited scope in what it can store. The same
data is simple, such that it can creatively be used for multiple use cases. Reduce won't do much
without your knowledge. Beyond internal backups, it won't change anything until you take some
action. This means you won't have to worry about what happens when you're not looking. Because of
this Reduce can be compared to a physical management system. Stored notes and documents won't just
move on their own. And everything has a clear and organized location.

The second goal is intent. Every change made to items is always intentional. It should not be
possible that something unexpected happens without your knowledge. No data is ever deleted until it
is verified by the user. And beyond intrinsic meta information like "last updated" no additional
fields are changed beyond what the user changed. Opening an item doesn't change it, and no
traditional notifications exist as those can also be read. This does mean that the system is poll
based. In other words, it is a requirement that the user periodically check the tool if they don't
want to miss time-based constraints.

## Quick setup

Before working on Reduce, a few prerequisites are required. Make sure you have installed the
following:

- The Rust tool chain and Cargo[^irust]
- Docker[^idocker]

Before development, run the following command:

```bash
docker compose up
```

This will start 2 docker containers:

- A Postgres server exposed on port 5432. It comes with a default database called "reduce_dev".
- An Adminer server, which can be used to directly view the database. You can access it in the
  browser at "localhost:8080".

The Postgres container is required for the compilation to work, as SQLx uses the database at
compile time to validate queries. After that run the following command to run the server:

```bash
cargo run
```

You can now access the server at "localhost:3000".

## Deployment

No official binaries or containers are provided at this point in time. It's not recommended to run
any production build of Reduce at this time, as it is in very early development.

First make sure you have the Rust toolchain installed. Refer to the quick setup section for more
information.

Build the project in release mode, by running:

```bash
cargo build --release
```

Then it's time to set up the server. How this is done depends on where and how you host the server.
Before you set up the server itself, make sure you have a postgres database available. There are
several ways to do this, I suggest you do your own research. Chances are that your hosting provider
already has a database available.

To deploy the server itself, copy the resulting binary, located at `target/release/reduce`, to your
deployment server. The server requires the following environment variables to be set:

```bash
DATABASE_URL=postgres://user:password@domain:5432/database_name
```

Replace the information with the information for your database. You should have either created
these yourself, or received them from your hosting provider, depending on how you set up the
database. If you are unsure as to what these are, please refer to the documentation for Postgres if
you set it up yourself, or ask your hosting provider for additional information.

Now make sure the server is started, preferably upon server restart, an you should have a working
deployment server. Migrations are embedded in the binary, and work automatically. So no need to
setup any tables manually.

[^irust]: <https://www.rust-lang.org/tools/install>
[^idocker]: <https://docs.docker.com/get-docker/>
