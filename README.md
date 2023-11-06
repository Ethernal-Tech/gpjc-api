# GPJC API

Rust wrapper API for using fork of google/private-join-and-compute protocol.

## Before running the API:

In order to run Private Join and Compute, you need to install Bazel, if you
don't have it already.
[Follow the instructions for your platform on the Bazel website.](https://docs.bazel.build/versions/master/install.html)

In order to run API, you need to install Rust, if you don't have it already.
[To install Rust follow instructions from this website.](https://www.rust-lang.org/tools/install)

Rust API is using MSSQL database, before running you need to install MSSSQL server if you don't have it already.
[Follow the instructions for your platform on the Microsoft webiste.](https://learn.microsoft.com/en-us/sql/database-engine/install-windows/install-sql-server?view=sql-server-ver16) 
When MSSQL server is installed create **gpjc_data** database.

For ease of use, Makefile is used for building and running the application. If you don't have it already you can install it following instructions from [this webiste for Windows](https://gnuwin32.sourceforge.net/packages/make.htm), or [from this webiste for Linux based systems](https://www.gnu.org/software/make/).

## How to run the API:

First build the project using:
```bash
make build
```
Using this command will build both the private-join-and-compute protocol and the Rust API code.

After this you can start API. Depending on your operating system you will use:
```bash
make run-linux password='<your-mssql-password>'
```
or
```bash
make run-windows
```

These commands will start gpjc server on local machine(localhost).

To run the program with different address(not the localhost) for gpjc server for Windows use:
```bash
cargo run --bin gpjc-api -- <address>
```
For Linux based systems:
```bash
cargo run --bin gpjc-api -- <address> <your-mssql-password>
```

To test out methods you can use(be aware of key duplication when running these commands):
- Start gpjc server:
```shell
curl -X POST -d '{"tx_id": "1", "policy_id": "1"}' -H "Content-type: application/json" http://localhost:9090/api/start-server
```
- Start gpjc client:
```shell
curl -X POST -d '{"tx_id": "1", "receiver": "M23", "to": "0.0.0.0:10501"}' -H "Content-type: application/json" http://localhost:9090/api/start-client
```
- Get proof from DB:
```shell
curl -X GET -d '{"tx_id": "1"}' -H "Content-type: application/json" http://localhost:9090/api/proof
```