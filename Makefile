run:
	cargo run --bin gpjc-api -- localhost

build:
	$(MAKE) -C private-join-and-compute
	cargo build --bin gpjc-api

release:
	cargo build --release
	cargo build --target x86_64-pc-windows-gnu --release

release-multiple-machines:
	cargo build --release --features=client
	cargo build --target x86_64-pc-windows-gnu --release --features=client