FROM rust:1.72 as builder

WORKDIR /build
COPY . .

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \
        clang \
        llvm \
        jq \
        gcc; \
    rm -rf /var/lib/apt/lists/*

RUN cd /build && cargo build --release


FROM debian:bookworm-20230612-slim
WORKDIR /app

COPY --from=builder \
  /usr/lib/x86_64-linux-gnu/libssl.so.* \
  /usr/lib/x86_64-linux-gnu/libcrypto.so.* \
  /usr/lib/x86_64-linux-gnu/
COPY --from=builder /build/target/release/axon /app/axon
COPY --from=builder /build/devtools /app/devtools

CMD /app/devtools/docker/docker-entrypoint.sh
