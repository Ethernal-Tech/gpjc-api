run:
	cargo run --bin gpjc-api -- localhost

build:
	$(MAKE) -C private-join-and-compute
	cargo build --bin gpjc-api

release:
	cargo build --release
	rustup target add x86_64-pc-windows-gnu
	cargo build --target x86_64-pc-windows-gnu --release --features=windows-build

release-multiple-machines:
	cargo build --release --features=client
	cargo build --target x86_64-pc-windows-gnu --release --features=client, windows-build