#!/bin/sh

set -ex

export RUST_LOG="info"

cargo build
sudo ip netns exec R \
  ./target/debug/mdns-repeater
