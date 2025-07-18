# RustSslFileServer

For a school course project

Using RustOpenSsl + Private Application Layer Protocol

## Run

First, you need to install the Rust and MySQL environments, then set up the MySQL database by referring to init.sql.

Next, generate the certificates using the following commands:

```bash
mkdir -p ssl
openssl genpkey -algorithm RSA -out ssl/key.pem -pkeyopt rsa_keygen_bits:2048
openssl req -new -x509 -key ssl/key.pem -out ssl/cert.pem -days 365 -subj "/CN=localhost" # Adjust "localhost" to other domains according to your situation
```

Don't forget to make a copy of cert.pem to the Client's directory.

Then, go to src/main.rs and modify MYSQL_URL to your MySQL connection string.

By the way you may need to make a dir for storing files.

```bash
mkdir -p storage
```

Once everything is ready, run:

```bash
DATABASE_URL="[YOUR_MYSQL_URL]" cargo run
```

## Client

https://github.com/Chisonline/rust_ssl_file_client