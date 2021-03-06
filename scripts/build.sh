#!/bin/bash

source $(dirname $0)/tool.sh

mkdir -pv $KIME_OUT

if [ -z "$KIME_MAKE_ARGS" ]; then
    KIME_MAKE_ARGS="-j4"
fi

if [ -z "$KIME_SKIP_ENGINE" ]; then
    KIME_SKIP_ENGINE=0
fi

set_release() {
    NEED_STRIP=1
    TARGET_DIR=./target/release
    _KIME_CARGO_ARGS="--release"
    _KIME_CMAKE_ARGS="-DCMAKE_BUILD_TYPE=Release"
}

set_debug() {
    NEED_STRIP=0
    TARGET_DIR=./target/debug
    _KIME_CARGO_ARGS=""
    _KIME_CMAKE_ARGS="-DCMAKE_BUILD_TYPE=Debug"
}

cargo_build() {
    cargo build $_KIME_CARGO_ARGS $KIME_CARGO_ARGS "$@"
}

set_release

while getopts hrda opt; do
    case $opt in
        h)
            echo "build.sh"
            echo "-h: help"
            echo "-r: release mode(default)"
            echo "-d: debug mode"
            echo "-a: all immodules"
            exit 0
            ;;
        r)
            set_release
            ;;
        d)
            set_debug
            ;;
        a)
            KIME_CMAKE_ARGS="-DENABLE_GTK2=ON -DENABLE_GTK3=ON -DENABLE_GTK4=ON -DENABLE_QT5=ON -DENABLE_QT6=ON $KIME_CMAKE_ARGS"
            ;;
    esac
done

if [ "$KIME_SKIP_ENGINE" -eq "1" ]; then
    _KIME_CMAKE_ARGS="${_KIME_CMAKE_ARGS} -DUSE_SYSTEM_ENGINE=ON"
    echo Use system engine
else
    LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:${PWD}/${TARGET_DIR}"
    echo Build core...
    cargo_build -p kime-engine-capi
    echo Build check...
    cargo_build -p kime-check
    cp $TARGET_DIR/libkime_engine.so $KIME_OUT
    cp $TARGET_DIR/kime-check $KIME_OUT
fi

echo Build xim wayland...

cargo_build -p kime-xim -p kime-wayland

cp $TARGET_DIR/kime-xim $KIME_OUT
cp $TARGET_DIR/kime-wayland $KIME_OUT
cp src/engine/cffi/kime_engine.h $KIME_OUT
cp src/engine/cffi/kime_engine.hpp $KIME_OUT
cp LICENSE $KIME_OUT
cp -R res/* $KIME_OUT

mkdir -pv build/cmake

echo Build gtk qt immodules...

cd build/cmake

cmake ../../src $_KIME_CMAKE_ARGS $KIME_CMAKE_ARGS

make $KIME_MAKE_ARGS

cp lib/* $KIME_OUT

if [ $NEED_STRIP -eq "1" ]; then
    strip -s $KIME_OUT/* 2&>/dev/null || true
fi
