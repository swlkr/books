#!/bin/sh

set -euo pipefail

app="books"
server=sean@fitzroy

#################
#    COMPILE    #
#################

echo "Building frontend..."
tailwindcss --input tailwind.input.css --output static/tailwind.css --minify
echo "Frontend build succeeded!"

echo "Building backend..."
cargo build --release --target=x86_64-unknown-linux-musl

echo "Build succeeded!"

#################
#     COPY      #
#################
scp target/x86_64-unknown-linux-musl/release/$app $server:./$app/$app-new

cat $(pwd)/server.sh | ssh -T sean@fitzroy

exit 0
