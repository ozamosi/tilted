FROM rust:1.47-slim

ADD . /usr/src/tilted

WORKDIR /usr/src/tilted

RUN apt update && apt install -y pkg-config libbluetooth-dev libssl-dev && cargo build --release

CMD tilted -c /etc/tilted/tilted.conf
