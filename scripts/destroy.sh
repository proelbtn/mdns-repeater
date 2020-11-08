#!/bin/sh

set -ex

sudo ip netns delete R
sudo ip netns delete C
sudo ip netns delete S
