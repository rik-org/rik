FROM rust:1.66 as builder
WORKDIR /build
COPY . .
RUN apt update -y && apt install -y protobuf-compiler
RUN cargo build -p scheduler

FROM gcr.io/distroless/cc
COPY --from=builder /build/target/debug/scheduler .
CMD ["./scheduler"]