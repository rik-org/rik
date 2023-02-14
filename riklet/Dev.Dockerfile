FROM rust:1.66 as builder
# MUST ADD : --build-arg ssh_prv_key="$(cat ~/.ssh/id_rsa)" --build-arg ssh_pub_key="$(cat ~/.ssh/id_rsa.pub)"
# because of private repo in cargo.toml
# Or simply use `make build`
ARG ssh_prv_key
ARG ssh_pub_key

WORKDIR /build
COPY . .
RUN apt update -y && apt install -y protobuf-compiler
RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh && \ 
  echo "$ssh_prv_key" > /root/.ssh/id_rsa && echo "$ssh_pub_key" > /root/.ssh/id_rsa.pub && \
  chmod 600 /root/.ssh/id_rsa && chmod 600 /root/.ssh/id_rsa.pub && \
  ssh-keyscan github.com >> /root/.ssh/known_hosts
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build -p riklet

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