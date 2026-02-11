#!/bin/bash
# Fix ownership of mounted volumes (created as root on first mount)
sudo chown -R vscode:vscode /home/vscode/.claude
sudo chown -R vscode:vscode /home/vscode/.config/gh

# Ensure Claude Code official marketplace is registered (needed on fresh volumes)
claude plugin marketplace add anthropics/claude-plugins-official || true

# Install LSP plugins for Claude Code
claude plugin install rust-analyzer-lsp
claude plugin install typescript-lsp
