FROM rust:latest as builder

WORKDIR /build
COPY . .

RUN apt-get update
RUN apt-get install cmake clang llvm gcc -y
RUN cd /build && cargo build --release

FROM debian:bookworm-20211011-slim
WORKDIR /app

RUN apt-get update
RUN apt install -y libssl-dev
RUN apt install -y libc6-dev

COPY --from=builder /build/target/release/axon /app/axon
COPY --from=builder /build/devtools /app/devtools

CMD ./axon -c=/app/devtools/chain/config.toml -g=/app/devtools/chain/genesis.json -m=/app/devtools/chain/metadata.json


