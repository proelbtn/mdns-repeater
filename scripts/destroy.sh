#!/bin/sh

set -ex

sudo ip netns delete R
sudo ip netns delete C1
sudo ip netns delete C2
sudo ip netns delete C3
