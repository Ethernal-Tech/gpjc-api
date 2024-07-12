# GPJC API

Rust wrapper API for using fork of google/private-join-and-compute protocol.

## Before running the API:

In order to build Private Join and Compute, you need to install Bazel, if you
don't have it already.
[Follow the instructions for your platform on the Bazel website.](https://docs.bazel.build/versions/master/install.html)

In order to run API, you need to install Rust, if you don't have it already.
[To install Rust follow instructions from this website.](https://www.rust-lang.org/tools/install)

For ease of use, Makefile is used for building and running the application. If you don't have it already you can install it following instructions from [this webiste for Windows](https://gnuwin32.sourceforge.net/packages/make.htm), or [from this webiste for Linux based systems](https://www.gnu.org/software/make/).

## How to run the API:

First build the project using:
```bash
make build
```
Using this command will build both the private-join-and-compute protocol and the Rust API code.

After this you can start API. Depending on your operating system you will use:
```bash
make run-linux
```
or
```bash
make run-windows
```

These commands will start gpjc server on local machine(localhost).

---

In case of running the program on multiple machines:

1. Use different version of [google/private-join-and-compute](https://github.com/Ethernal-Tech/private-join-and-compute/tree/multiple-machines) intended for use on different machines
2. Edit `.env` file and set `BACKEND_API_ADDRESS` value to your backend server address
3. Edit `.env` file and set `DNS_NAME` value to DNS name of the machine that runs it (for example client1.ethernal.com) 

To run the program with different address(not the localhost) for gpjc server for Windows use:
```bash
cargo run --bin gpjc-api -- <address>
```
For Linux based systems:
```bash
cargo run --bin gpjc-api -- <address>
```

---

To test out methods you can use(be aware of key duplication when running these commands):
- Start gpjc server:
```shell
curl -X POST -d '{"compliance_check_id": "1", "policy_id": "1"}' -H "Content-type: application/json" http://localhost:9090/api/start-server
```
- Start gpjc client:
```shell
curl -X POST -d '{"compliance_check_id": "1", "policy_id": "1", "participants": ["Comapny A", "Comapny B"], "to": "0.0.0.0:10501"}' -H "Content-type: application/json" http://localhost:9090/api/start-client
```
