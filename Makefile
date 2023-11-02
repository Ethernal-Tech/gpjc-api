run-linux:
	cargo run --bin gpjc-api -- localhost $(password)

run-windows:
	cargo run --bin gpjc-api -- localhost

build:
	$(MAKE) -C private-join-and-compute
	cargo build --bin gpjc-api

clean:
	rm -rf Cargo.lock
	cargo clean

release:
	cargo build --release
	rustup target add x86_64-pc-windows-gnu
	cargo build --target x86_64-pc-windows-gnu --release

release-multiple-machines:
	cargo build --release --features=client
	cargo build --target x86_64-pc-windows-gnu --release --features=client