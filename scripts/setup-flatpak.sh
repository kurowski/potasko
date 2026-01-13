#!/bin/bash
# Install Flatpak development dependencies on-demand
# Run this script when you need to build Flatpak packages locally

set -e

echo "Installing Flatpak build dependencies..."

# Install system packages
sudo apt-get update
sudo apt-get install -y \
    flatpak \
    flatpak-builder \
    python3-pip \
    python3-aiohttp \
    python3-toml

# Install Python tools for Flatpak dependency vendoring
pip3 install --break-system-packages tomlkit flatpak-node-generator

# Add Flathub remote
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

echo "Installing Flatpak runtimes (this may take a while)..."

# Install required runtimes and SDKs
flatpak install -y --user flathub \
    org.flatpak.Builder \
    org.gnome.Platform//48 \
    org.gnome.Sdk//48 \
    org.freedesktop.Sdk.Extension.rust-stable//24.08 \
    org.freedesktop.Sdk.Extension.node22//24.08

echo ""
echo "Flatpak development environment ready!"
echo "You can now build with: flatpak-builder --user --install --force-clean build net.kurowski.potasko.yml"
