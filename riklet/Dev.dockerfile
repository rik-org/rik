FROM rust:1.66 as builder
WORKDIR /build
COPY . .
RUN apt update -y && apt install -y protobuf-compiler
RUN cargo build -p riklet

FROM debian:stable-slim
RUN mkdir /app
WORKDIR /app
COPY ./firecracker /usr/bin/firecracker
RUN chmod +x /usr/bin/firecracker
COPY ./vmlinux.bin ./vmlinux.bin
COPY ./rootfs.ext4 ./rootfs.ext4
COPY ./config.json ./config.json
COPY --from=build /build/target/debug/riklet .
RUN apt update && apt install -y skopeo runc umoci ca-certificates
COPY --from=builder /build/target/debug/riklet .
CMD ["./riklet"]