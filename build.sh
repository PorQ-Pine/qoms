#!/bin/bash

cd qoms/

# In theory not needed but I'm scared
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="../../rootfs_sysroot/sysroot"
# In theory not needed but I'm scared
export PKG_CONFIG_PATH="../../rootfs_sysroot/sysroot/usr/lib/aarch64-linux-gnu/pkgconfig"
export RUSTFLAGS="-L ../../rootfs_sysroot/sysroot/usr/lib64"

cargo zigbuild --release --target aarch64-unknown-linux-gnu

