FROM rust:1.42 as build
COPY src/ ./src/
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo build --release
FROM busybox:latest
COPY --from=build ./target/release/galerians ./galerians
CMD ["./galerians"]
