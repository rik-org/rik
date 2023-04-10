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
    local size="200"
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
packages="udev systemd-sysv iproute2 nginx"
dirs="bin etc home lib lib64 opt root sbin usr"

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

for d in dev proc run sys var; do mkdir /rootfs/${d}; done
umount /rootfs
EOF

    info "Move built rootfs into ${rootfs_output}"
    mv ${rootfs_dir_host}/rootfs.ext4 ${rootfs_output}
    info "DONE"
}

parse_and_run_command $@