# mdns-repeater

Fork of https://bitbucket.org/geekman/mdns-repeater, with Rust.

## Usage

```
mdns-repeater 0.1.0
proelbtn <contact@proelbtn.com>
Multicast DNS Repeater

USAGE:
    mdns-repeater --config-file <CONFIG>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --config-file <CONFIG>    Config file [default: config.yaml]
```

## config.yaml

```yaml
interfaces:
  - name: tap0
    address: 172.16.0.0/24
  - name: tap1
    address: 172.16.1.0/24
```

