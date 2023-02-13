# Create a microVM with firecracker from Gateway until Riklet

## Setup

```bash
git clone git@github.com:rik-org/rik.git
git clone git@github.com:polyxia-org/gateway.git

# add vmlinux.bin and rootfs
curl https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin -o ./rik/vmlinux.bin
curl https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/rootfs/bionic.rootfs.ext4 -o ./rik/rootfs.ext4

# build and run rik

cd rik

git checkout develop

docker rm -f rik-scheduler
docker rm -f rik-controller
docker rm -f rik-worker

sudo docker compose up -d --build

exit # exit from root

# install and use java 17
curl -s "https://get.sdkman.io" | bash
source "/root/.sdkman/bin/sdkman-init.sh"
sdk install java 17.0.5-librca
sdk use java 17.0.5-librca

# run the gateway

cd gateway

git checkout 1-add-an-endpoint-into-the-controller-to-invoke-a-function

./mvnw spring-boot:run
```

## Create a workload

```bash
curl -X POST \
  http://localhost:8080/create \
  -H 'Content-Type: application/json' \
  -d '{
   "api_version": "v0",
   "kind": "function",
   "name": "demoff",
   "replicas": 1,
   "spec": {
       "containers": [
           {
               "name": "alpine",
               "image": "alpine:latest"
           }
       ]
   }
}'
```

**Becareful, the workload's name must be unique.**

A workload id will be returned.

## Create an instance

```bash
curl -X POST \
  http://localhost:8080/invoke \
  -H 'Content-Type: application/json' \
  -d '{
    "workload_id": "WORKLOAD_ID_RETURNED_BY_PREVIOUS_COMMAND"
    }'
```