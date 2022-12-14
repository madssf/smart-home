FROM rust:1.63 AS chef
RUN cargo install cargo-chef --locked
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11 as runtime
WORKDIR app
EXPOSE 8081

COPY --from=builder /app/configuration /app/configuration
COPY --from=builder /app/target/release/rust_home ./target/release/rust_home

CMD ["/app/target/release/rust_home"]
