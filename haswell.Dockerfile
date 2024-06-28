FROM rust:latest as builder
WORKDIR /usr/src/rusted-fbt
COPY . .
RUN env RUSTFLAGS="-C target-cpu=haswell" cargo install --path .

FROM debian:buster-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rusted-fbt /usr/local/bin/rusted-fbt
run apt update -y
run apt install ca-certificates -y
run apt update -y
# This is the only dependancie missing as far as I can tell which is great
run apt install libssl-dev -y
CMD ["rusted-fbt"]