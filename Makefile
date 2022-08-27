build:
	cargo run

run_docker:
	docker run --rm -it -v $(shell pwd):/rust rust /bin/sh -c "cd rust && cargo build" && ./target/debug/digger_rs

docker:
	docker run --rm -it -v $(shell pwd):/rust rust

# premise: in ↑docker
release:
	rustup target add wasm32-unknown-unknown && \
	cargo install wasm-bindgen-cli && \
	cargo build --release --target wasm32-unknown-unknown && \
	wasm-bindgen target/wasm32-unknown-unknown/release/digger_rs.wasm --out-dir wasm --no-modules --no-typescript

fmt:
	cargo fmt && \
	cargo clippy

fix:
	cargo fix
