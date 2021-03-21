FROM rust:1.50-slim AS builder

RUN apt update && apt install -y pkg-config libssl-dev

WORKDIR /usr/src/tilted

COPY Cargo.toml Cargo.lock /usr/src/tilted

# Build all dependencies, on their own layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release

ADD src /usr/src/tilted/src

RUN cargo build --release && cp target/release/tilted . && rm -rf target

FROM debian:stable-slim AS runner

COPY --from=builder /usr/src/tilted/src /usr/bin/tilted

CMD tilted -c /etc/tilted/tilted.conf
