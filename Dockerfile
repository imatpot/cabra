FROM rust:1.94.0-alpine AS builder

RUN apk add --no-cache \
    build-base

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

RUN cargo build --release && \
    rm -rf src

COPY src ./src/

RUN touch src/main.rs && \
    cargo build --release && \
    chmod +x "target/release/cabra"

FROM scratch AS final

COPY --from=builder /app/target/release/cabra /usr/local/bin/cabra

ENTRYPOINT [ "cabra" ]
