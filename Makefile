.PHONY: fetch lint build local-image

# Fetches new dependencies from cargo registries
fetch:
	cargo fetch --locked 

# Lints rust code via fmt and clippy
lint:
	cargo fmt
	cargo clippy --fix --allow-dirty

# Checks formatting, tests code, and builds binaries
build:
	cargo fmt -- --check
	cargo test --locked
	cargo build --locked --release --all

# Build the and load the docker image locally
local-image:
	docker buildx build \
		--load \
		--tag open-sauced-repo-query:latest .

# First builds the image and then starts the docker compose configuration
up: local-image
	docker-compose up
