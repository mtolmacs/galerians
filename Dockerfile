FROM rust:1.23 as build
COPY src/ ./
RUN cargo build --release
FROM gcr.io/distroless/static-debian10
COPY --from=build ./target/release/galera_dynamic_cluster .
ENTRYPOINT ["./galera_dynamic_cluster"]
