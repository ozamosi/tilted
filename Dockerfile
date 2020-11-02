FROM rust:1.47-slim

RUN apt update && apt install -y pkg-config libssl-dev && apt clean

WORKDIR /usr/src/tilted

ADD Cargo.toml /usr/src/tilted/Cargo.toml

# Build all dependencies, on their own layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release

ADD src /usr/src/tilted/src

RUN cargo build --release

CMD tilted -c /etc/tilted/tilted.conf
