FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y \
    mold clang \
    libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev \
    libxi6 libxrandr-dev libxcursor-dev \
    libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers