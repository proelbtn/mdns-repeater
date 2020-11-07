#!/bin/sh

set -ex

sudo ip netns add R
sudo ip netns add C1
sudo ip netns add C2
sudo ip netns add S1

sudo ip link add name net0 netns R type veth peer name net0 netns C1
sudo ip link add name net1 netns R type veth peer name net0 netns C2
sudo ip link add name net2 netns R type veth peer name net0 netns S1

sudo ip -n R link set lo up
sudo ip -n R link set net0 up
sudo ip -n R link set net1 up
sudo ip -n R link set net2 up
sudo ip -n R addr add 10.0.0.254/24 dev net0
sudo ip -n R addr add 10.0.1.254/24 dev net1
sudo ip -n R addr add 10.0.2.254/24 dev net2

sudo ip -n C1 link set lo up
sudo ip -n C1 link set net0 up
sudo ip -n C1 addr add 10.0.0.1/24 dev net0

sudo ip -n C2 link set lo up
sudo ip -n C2 link set net0 up
sudo ip -n C2 addr add 10.0.1.1/24 dev net0

sudo ip -n S1 link set lo up
sudo ip -n S1 link set net0 up
sudo ip -n S1 addr add 10.0.2.1/24 dev net0
