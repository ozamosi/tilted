FROM rust:1.52-slim-buster AS builder

WORKDIR /usr/src/tilted

COPY Cargo.toml Cargo.lock /usr/src/tilted/

# Build all dependencies, on their own layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release

COPY src src

RUN touch src/main.rs && cargo build --release && cp target/release/tilted . && rm -rf target

FROM debian:stable-slim AS runner

COPY --from=builder /usr/src/tilted/tilted /usr/bin/tilted

CMD tilted -c /etc/tilted/tilted.conf
