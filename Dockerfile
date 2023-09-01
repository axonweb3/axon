FROM rust:1.72 as builder

WORKDIR /build
COPY . .

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \
        clang \
        llvm \
        gcc; \
    rm -rf /var/lib/apt/lists/*

RUN cd /build && cargo build --release

# TODO: update the parent image
FROM debian:bookworm-20211011-slim
WORKDIR /app

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        libssl-dev \
        libc6-dev \
        ca-certificates; \
     rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/axon /app/axon
COPY --from=builder /build/devtools /app/devtools

CMD /app/devtools/docker/docker-entrypoint.sh
