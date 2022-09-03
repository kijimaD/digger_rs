run_docker:
	docker run --rm -it -v $(shell pwd):/src --workdir /src rust /bin/sh -c "cargo build" && ./target/debug/digger_rs

docker:
	docker run --rm -it -v $(shell pwd):/src --workdir /src rust

# premise: in docker
wasm-build:
	rustup target add wasm32-unknown-unknown && \
	cargo install wasm-bindgen-cli && \
	cargo build --release --target wasm32-unknown-unknown && \
	wasm-bindgen target/wasm32-unknown-unknown/release/digger_rs.wasm --out-dir wasm --no-modules --no-typescript

linux-build:
	cargo install cross && \
	rustup target add x86_64-unknown-linux-gnu && \
	cross build --release --target x86_64-unknown-linux-gnu

windows-build:
	cargo install cross && \
	rustup target add x86_64-pc-windows-gnu && \
	cross build --release --target x86_64-pc-windows-gnu

mac-build:
	docker run --rm \
	    --volume "${PWD}":/src \
	    --workdir /src \
	    joseluisq/rust-linux-darwin-builder:1.63.0 \
	    sh -c "cargo build --release --target x86_64-apple-darwin"

fmt:
	cargo fmt && \
	cargo clippy

fix:
	cargo fix
