FROM rust:latest as builder
WORKDIR /usr/src/rusted-fbt
COPY . .
RUN env RUSTFLAGS="-C target-cpu=westmere" cargo install --path .

FROM ubuntu:23.10
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rusted-fbt /usr/local/bin/rusted-fbt
RUN apt update -y
RUN apt install ca-certificates -y
RUN apt update -y
# This is the only dependancie missing as far as I can tell which is great
RUN apt install libssl-dev -y
RUN apt autoremove -y
CMD ["rusted-fbt"]