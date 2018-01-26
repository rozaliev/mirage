#!/usr/bin/env bash

docker run -ti --rm \
        -v "$PWD":/code:delegated \
        -v "$PWD/.linux-target":/code/target:delegated \
        -v "$PWD/.linux-cargo-cache/git":/usr/local/cargo/git:delegated \
        -v "$PWD/.linux-cargo-cache/registry":/usr/local/cargo/registry:delegated \
        -w /code rustlang/rust:nightly \
        /code/hack/run-tests.sh
