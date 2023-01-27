FROM rust:1.66 as builder
WORKDIR /build
COPY . .
RUN apt update -y && apt install -y protobuf-compiler 
RUN cargo build -p riklet

FROM debian:stable-slim
RUN mkdir /app
WORKDIR /app

# # RUN chmod +x /app/firecracker
COPY ./vmlinux.bin ./vmlinux.bin
COPY ./rootfs.ext4 ./rootfs.ext4
COPY ./config.json ./config.json
# # COPY . .
# # COPY ./README.md ./README.md
COPY ./firecracker ./firecracker
# RUN curl -L https://github.com/firecracker-microvm/firecracker/releases/download/v1.1.4/firecracker-v1.1.4-x86_64.tgz | tar -xz \
#     && mv firecracker-v1.1.4-x86_64 /app/firecracker
# COPY ./data.zip ./data.zip
RUN apt update && apt install -y skopeo runc umoci ca-certificates
# RUN unzip data.zip
COPY --from=builder /build/target/debug/riklet .
CMD ["./riklet"]