FROM rust:1.97.1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang cmake pkg-config libclang-dev libfontconfig1-dev libfreetype6-dev \
    libx11-dev libx11-xcb-dev libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxcb-xkb-dev libxcb-render0-dev libxcb-randr0-dev libxcb-sync-dev \
    libxkbcommon-dev libxkbcommon-x11-dev libxcb-imdkit-dev libwayland-dev \
    libasound2-dev libgtk-3-dev libssl-dev libvulkan-dev \
    xvfb xauth x11-utils xdotool imagemagick mesa-vulkan-drivers \
    libgl1-mesa-dri fonts-dejavu-core fonts-noto-cjk
RUN rustup component add clippy rustfmt
RUN apt-get update && apt-get install -y --no-install-recommends x11-apps
# Bookworm標準Mesa22ではGPUIのVulkan描画が黒くなるため、実描画確認済みの版を使う。
RUN printf '%s\n' 'deb http://deb.debian.org/debian bookworm-backports main' > /etc/apt/sources.list.d/kcu-backports.list
RUN apt-get update && apt-get install -y --no-install-recommends -t bookworm-backports mesa-vulkan-drivers=25.0.7-2~bpo12+1
RUN apt-get update && apt-get install -y --no-install-recommends openbox
ENV LIBGL_ALWAYS_SOFTWARE=1
WORKDIR /workspace
