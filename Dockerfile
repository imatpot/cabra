FROM rust:1.95.0-alpine AS rs

WORKDIR /app
RUN apk add --no-cache build-base
RUN cargo install cargo-chef

FROM rs AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rs AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM scratch AS final

COPY --from=builder /app/target/release/cabra /usr/local/bin/cabra
ENTRYPOINT [ "cabra" ]
