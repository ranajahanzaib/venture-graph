.PHONY: setup dev build test lint format seed import clean
setup:
	npm install --global pnpm@latest
	pnpm install

dev:
	docker compose up --build

build:
	cargo build --workspace
	pnpm build

test:
	cargo test --workspace
	pnpm test

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	pnpm lint

format:
	cargo fmt --all

seed:
	@echo "Synthetic demo data is inserted automatically by the first API migration."

import:
	@test -n "$(CSV)" || (echo "Usage: make import CSV=/path/to/sample.csv" && exit 2)
	cargo run -p venturegraph-api --bin venturegraph-import -- "$(CSV)"

clean:
	cargo clean
	rm -rf apps/web/dist
