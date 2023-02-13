get_kernel_and_rootfs:
	curl https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin -o ./vmlinux.bin
	curl https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/rootfs/bionic.rootfs.ext4 -o ./rootfs.ext4

build:
	docker build -t test --build-arg ssh_prv_key="`cat ~/.ssh/id_rsa`" --build-arg ssh_pub_key="`cat ~/.ssh/id_rsa.pub`" -f ./riklet/Dev.Dockerfile .

run:
	docker compose up -d
	echo "/!\ Gateway repo must be cloned in the same directory as this repo"
	cd ../gateway && ./mvnw spring-boot:run