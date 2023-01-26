FROM rust:1.66 as builder
WORKDIR /build
COPY . .
RUN apt update -y && apt install -y protobuf-compiler
RUN cargo build -p riklet

FROM debian:stable-slim
RUN apt update && apt install -y skopeo runc umoci ca-certificates
COPY --from=builder /build/target/debug/riklet .
CMD ["./riklet"]
