@echo off

cd /D "%~dp0" || exit 1

mkdir build

pushd starter || exit 1

cargo build --release || exit 1

popd || exit 1

copy starter\target\release\starter.exe build\clash-multiplatform.exe || exit 1

rmdir /S /Q build\jre

jlink --add-modules java.base,java.desktop,java.logging ^
  --output build\jre ^
  --ignore-signing-information ^
  --no-header-files --no-man-pages --strip-debug ^
  --verbose || exit 1

pushd build || exit 1

zip -r bundle.zip * || exit 1

popd || exit 1

move build\bundle.zip bundle.zip || exit 1
