#!/bin/bash

wget https://nodejs.org/dist/v22.23.2/node-v22.23.2-linux-x64.tar.xz
tar -Jxvf ./node-v22.23.2-linux-x64.tar.xz
export PATH=$(pwd)/node-v22.23.2-linux-x64/bin:$PATH
npm install pnpm -g

rustup target add "$INPUT_TARGET"
rustup toolchain install --force-non-host "$INPUT_TOOLCHAIN"

apt-get update
apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev libxcb1 libxrandr2 libdbus-1-3 xdg-utils

bash .github/actions/build.sh
