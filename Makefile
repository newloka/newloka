PROJECT_NAME := newloka
RUSTC := rustc
CARGO := cargo

.PHONY: all build test fmt lint clean docker serve docs

all: fmt lint test build

build:
	$(CARGO) build --release

test:
	$(CARGO) test --all

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

clean:
	$(CARGO) clean

docker:
	docker build -t $(PROJECT_NAME):latest .

docker-compose:
	docker-compose up -d

serve:
	$(CARGO) run --bin newloka-server -- --bind 127.0.0.1:8080

cli:
	$(CARGO) run --bin newloka-cli -- serve --bind 127.0.0.1:8080

docs:
	$(CARGO) doc --no-deps --open

install:
	$(CARGO) install --path newloka_cli
	$(CARGO) install --path newloka_server

