FROM rust:1.75-slim-bookworm AS builder

RUN apt-get update && apt-get install -y libsqlite3-dev pkg-config

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/newloka-server /usr/local/bin/newloka-server
COPY --from=builder /app/target/release/newloka /usr/local/bin/newloka

ENV NEWLOKA_DB_PATH=/data/newloka.db
ENV NEWLOKA_BIND_ADDR=0.0.0.0:8080
ENV NEWLOKA_TIER=t2

VOLUME ["/data"]
EXPOSE 8080

CMD ["newloka-server"]
