#!/bin/bash
# Devcontainer startup script

# Start D-Bus (required for flatpak operations)
sudo mkdir -p /var/run/dbus
sudo dbus-daemon --system --fork

# Start Radicale CalDAV server
sudo service radicale start
