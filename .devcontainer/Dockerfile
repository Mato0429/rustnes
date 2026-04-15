FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

RUN apt-get update \
&& apt-get install -y \
    x11-apps \
    build-essential \
    curl \
    wget \
    file \
    git \
    sudo \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
&& curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
&& apt-get install -y nodejs \
&& rm -rf /var/lib/apt/lists/* \
&& corepack enable && corepack prepare pnpm@latest --activate
