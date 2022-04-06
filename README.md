Modified based on the code in Chapter 14 https://bfnightly.bracketproductions.com/chapter_14.html

![スクリーンショット 2022-03-26 11-58-15](https://user-images.githubusercontent.com/11595790/160222341-af82b4c9-89c0-48ff-858e-5ea09b123dbf.png)

## Feature

- Keyboard oriented
- Symol encounter & Front view battle

## operation

- WASD: Movement
- G: Get item
- I: Open inventory
- T: Drop item
- R: Remove equipment

## Docker

development build

```shell
$ cargo run
```

release build

```shell
$ docker run --rm -it -v $(pwd):/rust rust
```

```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cd rust
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/digger_rs.wasm --out-dir wasm --no-modules --no-typescript
```
