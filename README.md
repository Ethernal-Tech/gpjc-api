# GPJC API

Rust wrapper API for using fork of google/private-join-and-compute protocol.

## How to run the API:

In order to run Private Join and Compute, you need to install Bazel, if you
don't have it already.
[Follow the instructions for your platform on the Bazel website.](https://docs.bazel.build/versions/master/install.html)

Rust API is using MSSQL database, before running you need to create **gpjc_data** database.

First build the project using:
```bash
make build
```
Using this command will build both the private-join-and-compute protocol and the Rust API code.

After this you can start API with:
```bash
make run
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