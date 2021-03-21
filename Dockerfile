FROM rust:1.50-slim

RUN apt update && apt install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*;

WORKDIR /usr/src/tilted

ADD Cargo.toml /usr/src/tilted/Cargo.toml

# Build all dependencies, on their own layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release

ADD src /usr/src/tilted/src

RUN cargo build --release && cargo install --bins --path . && rm -rf target

CMD tilted -c /etc/tilted/tilted.conf
