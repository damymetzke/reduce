# Reduce

Reduce is a web based application meant to reduce the complexity of tasks.
This is interpreted in 2 ways. The first of which is that it uses an interface which
attempts to reduce the information it shows you. All information is targeted to what you need to know,
and usage is targeted towards "learning" the application by repeated usage. In practice this means less
flashy styling, limited dynamic content, and clever ways to interpret data.
A good example is how projects works. Projects are abstract categorizations of work, and support hierarchies.
Dots can be used when entering projects in most cases. So you can create a bunch of projects just by inputting something
like "foo.bar.baz".

The second way is in how it models your tasks. Everything in Reduce is information that needs to be
processed. This can be seen as a list of steps you need to take. Every step you take you reduce the
amount of work by a little bit. Similar to how the functional higher order function reduce[^reducehigerorderfunction] works.

> [!CAUTION]
> Reduce is in a development version. Absolutely no reasonable security processes are in place. Run
> at your own risk.

## Content

- [Quick setup](#quick-setup)
- [Deployment](#deployment)

## Quick setup

Before working on Reduce, a few prerequisites are required. Make sure you have installed the
following:

- The Rust tool chain and Cargo[^irust]
- sqlx-cli[^isqlxcli]
- Docker[^idocker]
- Node.js[^inode]

While not strictly required, the following programs are recommended for use during development:

- cargo-watch[^icargowatch]
- mprocs[^improcs]

It's recommended to run the project using mprocs:

```bash
mprocs
```

This will start all required programs within a single terminal window. Please reference
mprocs itself for further information about how to interact with the multiple windows.

If you prefer to manually manage the processes, you should make sure to run the following commands in separate sessions:

```bash
docker compose up
npm run tailwind
# Run cargo directly, requires manual restarting
cargo r
# Use cargo-watch for automatic restarts
cargo watch -x r
```

> [!NOTE]
> This doesn't include clippy, which you may want to run manually.

There is one more step required before the build will succeed. For that run the following command:

```bash
sqlx migrate run
```

This will run migrations, which is required for the build process to succeed.

The 3 commands are all required to run the project as a whole. Docker compose is used for the
database. This will run 2 containers, one is a Postgres database, the other is an adminer instance.
The latter can be used to manually inspect the database, and can be accessed on port 8080. Note that
this is purely meant for development. The credentials for the database in development mode are
"user" and "password".

The tailwind command in npm is used to watch changes in the `templates/` directory, and then
rebuild the resulting css file which is then served from within the application.

Cargo r runs the server itself. It requires that the database is online, As mentioned already, SQLx
uses the actual database connection during compilation. If you reset the database, make sure to run
`sqlx migrate run` again. The CSS is compiled into the binary itself, so you also need to compile
the tailwind css sheet.

## Deployment

No official binaries or containers are provided at this point in time. It's not recommended to run
any production build of Reduce at this time, as it is in very early development.

First make sure you have the Rust tool chain installed. Refer to the quick setup section for more
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

[^reducehigerorderfunction]: <https://en.wikipedia.org/wiki/Fold_(higher-order_function)>
[^irust]: <https://www.rust-lang.org/tools/install>
[^isqlxcli]: <https://lib.rs/crates/sqlx-cli>
[^idocker]: <https://docs.docker.com/get-docker/>
[^inode]: <https://docs.docker.com/get-docker/>
[^icargowatch]: <https://github.com/watchexec/cargo-watch>
[^improcs]: <https://github.com/pvolok/mprocs>
