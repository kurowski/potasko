#!/bin/bash
# Fix ownership of mounted volumes (created as root on first mount)
sudo chown -R vscode:vscode /home/vscode/.claude
sudo chown -R vscode:vscode /home/vscode/.config/gh
