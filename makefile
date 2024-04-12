setup.project:
	poetry install --all-extras

test.unit:
	cargo test -- --test-threads=1

format:
	cargo fmt

lints:
	cargo clippy --workspace --all-targets -- -D warnings

