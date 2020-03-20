FROM rust:1.42 as build
COPY src/ ./
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo build --release
FROM gcr.io/distroless/static-debian10
COPY --from=build ./target/release/galerians .
ENTRYPOINT ["./galerians"]
