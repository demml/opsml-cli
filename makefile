
test.unit:
	cargo test -- --test-threads=1

format:
	cargo fmt

lints:
	cargo clippy --workspace --all-targets -- -D warnings

setup.project:
	uv sync --all-extras --group dev
	uv run maturin develop