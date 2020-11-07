#!/bin/sh

set -ex

cargo build
sudo ip netns exec R \
  ./target/debug/mdns-repeater net0 net1 net2
