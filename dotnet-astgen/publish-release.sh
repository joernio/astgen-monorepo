#!/bin/bash
OS=$1
ARCH=$2
LATEST_TAG=`git describe --tags --abbrev=0 | cut -c2-`

echo "OS: $OS";
echo "Architecture: $ARCH";

if [[ -z "${NEXT_VERSION}" ]]; then
    echo "Version not set, defaulting to latest tag version $LATEST_TAG";
    export NEXT_VERSION=$LATEST_TAG
fi

RELEASE_DIR="./release"
OUTPUT_TARGET=""
if [[ $OS == "osx" ]]; then
    OUTPUT_TARGET="macos"
else 
    OUTPUT_TARGET=$OS
fi
if [[ $ARCH == arm* ]]; then
    OUTPUT_TARGET="$OUTPUT_TARGET-arm"
fi

OUTPUT_PATH="$RELEASE_DIR/$OUTPUT_TARGET"
TARGET="$OS-$ARCH"

dotnet publish -c Release -r $TARGET -o $OUTPUT_PATH
