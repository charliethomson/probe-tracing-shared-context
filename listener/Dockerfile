FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN SQLX_OFFLINE=true cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --bin listener

FROM debian:sid

RUN apt-get update && apt-get -y upgrade && apt-get install -y libssl-dev openssl ca-certificates curl
COPY --from=builder /usr/src/target/release/listener /usr/local/bin/listener
ENV PORT=8080
EXPOSE 8080

CMD ["listener"]
