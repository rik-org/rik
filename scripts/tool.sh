#!/usr/bin/bash

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
progname=$(basename $0)
resource_dir="scripts/resources"
source scripts/lib.sh

sub_default() {
  echo "Tool component for RIK project"
  echo ""
  echo "Usage: $progname <COMMAND> [option1] [option2]..."
  echo "Commands:"
  echo "    mkkernel                    Build a kernel that can be used in firecracker"
  echo "    mkrootfs                    Create a rootfs with sample services and stuff"
  echo "    run-firecracker             Run firecracker with kernel and rootfs built from this script"
  echo ""
  print_generic_options
}

# Build a kernel that can be used with VM workload
# Example: `./scripts/tool.sh mkkernel`
sub_mkkernel() {
    local linux_repo="${resource_dir}/linux-git"

    for cmd in bc bison git fakeroot; do
        check_prerequisite $cmd
    done

    if [ ! -d $linux_repo ]
    then
        info "Could not found linux kernel repository, cloning it locally..."
        git clone --depth 1 "https://github.com/torvalds/linux.git" -b "v5.10" $linux_repo
    fi

    info "Clone kernel config into linux kernel repository"
    cp ${resource_dir}/linux-config-x86_64 ${linux_repo}/.config

    pushd $linux_repo
    info "Build kernel on the system..."
    make vmlinux -j `nproc` 1> /dev/null
    info "Copy built kernel into vmlinux file"
    cp ./vmlinux ../../vmlinux
    popd
    info "Kernel available at scripts/vmlinux"
    info "DONE"
}

# Build a RootFS with some sample service that can be used for VM workload
# Example: `./scripts/tool.sh mkrootfs`
sub_mkrootfs() {
    # Size of the rootfs in MB
    local size="250"
    local origin_image="ubuntu:22.04"
    local rootfs_dir_host=/tmp/firecracker/
    local rootfs_dir_guest=/rootfs

    local rootfs_output="scripts/rootfs.ext4"

    for cmd in docker mkfs.ext4 mount; do
        check_prerequisite $cmd
    done

    if [ -f "${rootfs_output}" ]; then
        info "A previous rootfs.ext4 exist, do you want to override it ?"
        get_user_confirmation || exit 1
        rm -f ${rootfs_output}
    fi

    info "Create empty rootfs file of ${size}MB"
    mkdir -p ${rootfs_dir_host}
    dd if=/dev/zero of=${rootfs_dir_host}/rootfs.ext4 bs=1M count=${size}

    mkfs.ext4 -F ${rootfs_dir_host}/rootfs.ext4

    info "Run a docker image with ${origin_image} to setup the rootfs"
    docker run --privileged --rm -i \
        --env MNT_DIR="${rootfs_dir_guest}" \
        -v "${rootfs_dir_host}:/${rootfs_dir_guest}" ${origin_image} \
        bash -s <<'EOF'
packages="udev systemd-sysv iproute2 nginx wget"
dirs="bin etc home lib lib64 opt root sbin usr var/lib/nginx"

echo "Mount rootfs on the system"
mount ${MNT_DIR}/rootfs.ext4 /rootfs

echo "Install few packages"
apt-get update
apt-get install -y --no-install-recommends ${packages}

echo "Setup vm hostname"
echo "ubuntu-fc-uvm" > "/etc/hostname"

echo "Activate autologin to prevent to have to put credentials"
mkdir "/etc/systemd/system/serial-getty@ttyS0.service.d/"
cat <<CMD > "/etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf"
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root -o '-p -- \\u' --keep-baud 115200,38400,9600 %I $TERM
CMD

passwd -d root

echo "Copy rootfs into the mounted volume"
for d in $dirs; do tar c "/$d" | tar x -C /rootfs; done

for d in dev proc run sys var/log/nginx; do mkdir -p /rootfs/${d}; done
umount /rootfs
EOF

    info "Move built rootfs into ${rootfs_output}"
    mv ${rootfs_dir_host}/rootfs.ext4 ${rootfs_output}
    info "DONE"
}

# Runs a firecracker VM based on rootfs and kernel available in ./scripts
# 
# Note: You need to have sudo installed on your machine to run this command, as it needs to create
# a tap interface.
#
# Ref: https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md
sub_run-firecracker() {
    set -m
    local kernel="${PWD}/scripts/vmlinux"
    local rootfs="${PWD}/scripts/rootfs.ext4"
    local logs="${PWD}/scripts/firecracker.log"

    local api_socket="/tmp/rik-api.socket"
    # Network format:
    #  ip=<ip>::<gateway>:<netmask>:<hostname>:<device>:<autoconf>
    local boot_args="console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux quiet loglevel=0 ip=172.12.0.253::172.12.0.254:255.255.255.252::eth0:off"

    for cmd in "firecracker sudo"; do
        check_prerequisite $cmd
    done

    if [ -S $api_socket ]; then
        info "A previous firecracker instance is running, do you want to override it ?"
        get_user_confirmation || exit 1
        sudo rm -f $api_socket
    fi

    if [ ! -f $kernel ]; then
        info "Kernel not found, please build it first"
        exit 1
    fi

    if [ ! -f $rootfs ]; then
        info "RootFS not found, please build it first"
        exit 1
    fi

    if [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
        info "KVM is available"
    else
        info "KVM is not available, please enable it in your system (are you root?)"
        exit 1
    fi

    info "Run firecracker with kernel and rootfs"
    sudo firecracker --api-sock $api_socket &
    
    info "Configure logger..."
    echo "" > ${logs}
    sudo curl -X PUT -f --unix-socket "${api_socket}" \
    --data "{
        \"log_path\": \"${logs}\",
        \"level\": \"Debug\",
        \"show_level\": true,
        \"show_log_origin\": true
    }" \
    "http://localhost/logger"

    info "Configure kernel..."
    sudo curl -X PUT -f --unix-socket "${api_socket}" \
    --data "{
        \"kernel_image_path\": \"${kernel}\",
        \"boot_args\": \"${boot_args}\"
    }" \
    "http://localhost/boot-source"

    info "Configure rootfs..."
    sudo curl -X PUT -f --unix-socket "${api_socket}" \
    --data "{
        \"drive_id\": \"rootfs\",
        \"path_on_host\": \"${rootfs}\",
        \"is_root_device\": true,
        \"is_read_only\": false
    }" \
    "http://localhost/drives/rootfs"

    info "Configure network..."
    sudo curl -f --unix-socket "${api_socket}" -i \
    -X PUT 'http://localhost/network-interfaces/eth0' \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d '{
        "iface_id": "eth0",
        "guest_mac": "AA:FC:00:00:00:01",
        "host_dev_name": "rik-tap0"
        }'

    info "Give IP 172.12.0.254 to host TAP interface"
    sudo ip addr add 172.12.0.254/30 dev rik-tap0
    sudo ip link set rik-tap0 up

    # API requests are handled asynchronously, it is important the microVM has been
    # started before we attempt to SSH into it.
    sleep 0.015s

    info "Starting VM"
    sudo curl -X PUT -f --unix-socket "${api_socket}" \
    --data "{
        \"action_type\": \"InstanceStart\"
    }" \
    "http://localhost/actions" &

    info "Firecracker is running"
    fg %1


}

parse_and_run_command $@