# Built artifact will be in `app/target/wasm32-unknown-unknown/release/app.wasm`
.PHONY: build
build:
	cd app && cargo build --target wasm32-unknown-unknown --release


# Build project using wee_alloc global allocator
.PHONY: build-small
build-small:
	cd app && cargo build --target wasm32-unknown-unknown --release --features wee_alloc


# Run demo
.PHONY: run
run:
	cd host && cargo run -- ../app/target/wasm32-unknown-unknown/release/app.wasm


.PHONY: clean
clean:
	cd host && cargo clean
	cd app && cargo clean
