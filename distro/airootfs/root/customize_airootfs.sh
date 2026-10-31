#!/usr/bin/env bash
# Archiso customization hook, run inside the airootfs chroot while building.
#
# It makes the live medium usable out of the box:
#   - a populated pacman keyring, so `pacman -S anything` works immediately
#   - NetworkManager (Wi-Fi + Ethernet) enabled at boot
#   - systemd-timesyncd, so TLS certificates validate after a cold start
#   - the live user's shell aliases (wifi, fetch, yuna)
set -euo pipefail

echo "[yuna] setting default timezone to Cambodia (Asia/Phnom_Penh)"
ln -sf /usr/share/zoneinfo/Asia/Phnom_Penh /etc/localtime

echo "[yuna] initializing the pacman keyring"
pacman-key --init
pacman-key --populate archlinux

echo "[yuna] configuring live user (yuna:yuna) and root (root:yuna)"
if ! id -u yuna >/dev/null 2>&1; then
    useradd -m -g users -G wheel,audio,video,storage,optical,network,power -s /bin/bash yuna
fi
echo "yuna:yuna" | chpasswd
echo "root:yuna" | chpasswd

# Copy skeleton configs to live user home
if [[ -d /home/yuna ]]; then
    cp -rT /etc/skel /home/yuna
    chown -R yuna:users /home/yuna
fi

echo "[yuna] building the live initramfs"
mkinitcpio -P

echo "[yuna] enabling live services"
systemctl enable NetworkManager.service
systemctl enable systemd-timesyncd.service
systemctl enable systemd-resolved.service 2>/dev/null || true
systemctl enable ufw.service 2>/dev/null || true

# `iwd` is installed as an alternative to NetworkManager + wpa_supplicant.
# Do not enable it: running two supplicants at once breaks Wi-Fi.
systemctl disable iwd.service 2>/dev/null || true

echo "[yuna] done"
