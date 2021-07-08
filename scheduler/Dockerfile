# This Dockerfile aims to run the project on a distribution which is not
# supported by Runc, Skopeo and Umoci like macOS.
# This is only for development purposes.
FROM alpine:latest
RUN apk add --no-cache rust cargo protoc

WORKDIR /app

COPY src ./src/
COPY Cargo.* ./

RUN cargo build --release
RUN mv ./target/release/rik-scheduler /app/rik-scheduler

ENTRYPOINT ["/app/rik-scheduler", "-v"]
