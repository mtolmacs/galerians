FROM rust:1.42 as build
COPY src/ ./src/
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo build --release
FROM gcr.io/distroless/static-debian10:debug
COPY --from=build ./target/release/galerians ./galerians
ENTRYPOINT ["./galerians"]
