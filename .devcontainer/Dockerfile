FROM mcr.microsoft.com/devcontainers/rust:latest

# Link image to repository for GHCR permissions
LABEL org.opencontainers.image.source=https://github.com/kurowski/potasko

# Install Node.js 24
RUN curl -fsSL https://deb.nodesource.com/setup_24.x | bash - \
  && apt-get install -y nodejs

# Install system dependencies for Tauri desktop and Android development
RUN apt-get update && apt-get install -y \
  cmake \
  gh \
  openjdk-21-jdk \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  libayatana-appindicator3-dev \
  radicale \
  webkitgtk-webdriver \
  xdg-utils \
  xvfb \
  && rm -rf /var/lib/apt/lists/*

# Configure Radicale for development (no auth, use /tmp for storage)
RUN sed -i 's/^#\(hosts = localhost:5232\)/\1/' /etc/radicale/config && \
    sed -i 's/^#type = denyall/type = none/' /etc/radicale/config && \
    sed -i 's|^#filesystem_folder = /var/lib/radicale/collections|filesystem_folder = /tmp/radicale-data|' /etc/radicale/config

# Set up Java environment
ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
ENV PATH=$JAVA_HOME/bin:$PATH

# Install Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools

RUN mkdir -p $ANDROID_HOME/cmdline-tools \
  && cd $ANDROID_HOME/cmdline-tools \
  && CMDLINE_TOOLS_URL=$(curl -s https://developer.android.com/studio | grep -o 'https://dl.google.com/android/repository/commandlinetools-linux-[0-9]*_latest.zip' | head -1) \
  && wget -q $CMDLINE_TOOLS_URL \
  && unzip -q commandlinetools-linux-*_latest.zip \
  && mv cmdline-tools latest \
  && rm commandlinetools-linux-*_latest.zip

# Accept Android licenses and install SDK components
# NDK 28+ required for Android 15 16KB page alignment
RUN yes | sdkmanager --licenses || true \
  && sdkmanager "platform-tools" \
  && sdkmanager "platforms;android-35" \
  && sdkmanager "build-tools;35.0.0" \
  && sdkmanager "ndk;28.2.13676358"

# Set NDK environment variables
ENV NDK_HOME=$ANDROID_HOME/ndk/28.2.13676358
ENV ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/28.2.13676358
ENV ANDROID_NDK=$ANDROID_HOME/ndk/28.2.13676358

# Switch to vscode user for Rust installations
USER vscode

# Install rust-analyzer for IDE support
RUN rustup component add rust-analyzer

# Install Rust targets for Android
RUN rustup target add aarch64-linux-android \
  && rustup target add armv7-linux-androideabi \
  && rustup target add i686-linux-android \
  && rustup target add x86_64-linux-android

# Install Tauri CLI
RUN cargo install tauri-cli --version "^2.0.0" --locked

# Install tauri-driver for E2E testing
RUN cargo install tauri-driver

# Install sqlx-cli for database migrations
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Install Claude Code
RUN curl -fsSL https://claude.ai/install.sh | bash

# Set up Cargo environment
ENV PATH=/home/vscode/.cargo/bin:$PATH

# Switch back to root for any final setup
USER root

# Install Node.js package managers
RUN npm install -g pnpm yarn

# Install TypeScript Language Server and TypeScript
RUN npm install -g typescript-language-server typescript

# Ensure vscode user owns Android SDK
RUN chown -R vscode:vscode $ANDROID_HOME

# Switch back to vscode user as default
USER vscode

WORKDIR /workspace
