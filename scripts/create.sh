#!/bin/sh

set -ex

sudo ip netns add R
sudo ip netns add C
sudo ip netns add S

sudo ip link add name net0 netns R type veth peer name net0 netns C
sudo ip link add name net1 netns R type veth peer name net0 netns S

sudo ip -n R link set lo up
sudo ip -n R link set net0 up
sudo ip -n R link set net1 up
sudo ip -n R addr add 10.0.0.254/24 dev net0
sudo ip -n R addr add 10.0.1.254/24 dev net1

sudo ip -n C link set lo up
sudo ip -n C link set net0 up
sudo ip -n C addr add 10.0.0.1/24 dev net0

sudo ip -n S link set lo up
sudo ip -n S link set net0 up
sudo ip -n S addr add 10.0.1.1/24 dev net0
