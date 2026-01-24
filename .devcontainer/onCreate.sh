#!/bin/bash
# Fix ownership of mounted volumes (created as root on first mount)
sudo chown -R vscode:vscode /home/vscode/.claude
sudo chown -R vscode:vscode /home/vscode/.config/gh

# Install Rust Analyzer LSP plugin for Claude Code
claude plugin install rust-analyzer-lsp
claude plugin install typescript-lsp
