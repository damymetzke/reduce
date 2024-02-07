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
way Reduce is simple is in how you use it. Data has limited scope in what it can store. The same
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

> [!CAUTION]
> Reduce is in a development version. Absolutely no reasonable security processes are in place. Run
> at your own risk.

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
[^isqlxcli]: <https://lib.rs/crates/sqlx-cli>
[^idocker]: <https://docs.docker.com/get-docker/>
[^inode]: <https://docs.docker.com/get-docker/>
[^icargowatch]: <https://github.com/watchexec/cargo-watch>
[^improcs]: <https://github.com/pvolok/mprocs>
