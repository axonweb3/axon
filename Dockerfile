FROM rust:1.70 as builder

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


FROM debian:bookworm-20230612-slim
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
