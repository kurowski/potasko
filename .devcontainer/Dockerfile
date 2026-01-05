FROM mcr.microsoft.com/devcontainers/rust:latest

# Install Node.js 24
RUN curl -fsSL https://deb.nodesource.com/setup_24.x | bash - \
  && apt-get install -y nodejs

# Install system dependencies for Tauri desktop and Android development
RUN apt-get update && apt-get install -y \
  gh \
  openjdk-21-jdk \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  libayatana-appindicator3-dev \
  python3-pip \
  && rm -rf /var/lib/apt/lists/*

# Install Radicale for CalDAV development testing
RUN pip install --break-system-packages radicale

# Configure Radicale with test user (test/test)
RUN mkdir -p /etc/radicale && \
    echo -e "[server]\nhosts = 127.0.0.1:5232\n\n[auth]\ntype = htpasswd\nhtpasswd_filename = /etc/radicale/users\nhtpasswd_encryption = plain\n\n[storage]\nfilesystem_folder = /tmp/radicale-data\n\n[rights]\ntype = owner_only" > /etc/radicale/config && \
    echo "test:test" > /etc/radicale/users

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
RUN yes | sdkmanager --licenses || true \
  && sdkmanager "platform-tools" \
  && sdkmanager "platforms;android-35" \
  && sdkmanager "build-tools;35.0.0" \
  && LATEST_NDK=$(sdkmanager --list | grep "ndk;" | grep -v "ndk-bundle" | head -1 | awk '{print $1}') \
  && sdkmanager "$LATEST_NDK"

# Set NDK_HOME dynamically to the installed version (Tauri's recommended approach)
RUN echo 'export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)"' >> /etc/profile.d/android-ndk.sh

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

# Ensure vscode user owns Android SDK
RUN chown -R vscode:vscode $ANDROID_HOME

# Switch back to vscode user as default
USER vscode

WORKDIR /workspace
