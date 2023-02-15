#!/bin/bash

set -euo pipefail

TAP_ID="${1:-0}" # Default to 0
TAP_DEV="rik-${TAP_ID}-tap"

# Setup TAP device
MASK_SHORT="/30"
TAP_IP="${2:-169.254.0.1}"

ip link del "$TAP_DEV" 2> /dev/null || true
ip tuntap add dev "$TAP_DEV" mode tap
ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
ip link set dev "$TAP_DEV" up
