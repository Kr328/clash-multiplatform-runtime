#!/bin/bash

cd "$(dirname "$0")" || exit 1

pushd starter || exit 1

cargo build --release || exit 1

popd || exit 1

mkdir -p build

cp starter/target/release/starter build/clash-multiplatform || exit 1

rm -rf build/jre

jlink --add-modules java.base,java.desktop,java.logging \
  --output build/jre \
  --ignore-signing-information \
  --no-header-files --no-man-pages --strip-debug \
  --verbose || exit 1

pushd build || exit 1

zip -r bundle.zip ./* || exit 1

popd || exit 1

mv build/bundle.zip ./bundle.zip
