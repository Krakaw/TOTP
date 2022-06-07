wasm-web:
	wasm-pack build --release --target bundler -d wasm-web

wasm-node:
	wasm-pack build --release --target nodejs -d wasm-node

all:
	make wasm-web
	make wasm-node
