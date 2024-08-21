A lightweight virtual machine tool, for personal use.

only support Apple Silicon and macOS Sonoma

# Features
* create and run both Linux and MacOS VM
* run in GUI or detached mode

# Usage
```
Usage: vz <COMMAND>

Commands:
  ls          list vm status
  create      create vm
  run         run vm
  stop        stop vm
  ipsw        get macOS restore image ipsw url
  resize      increase disk image size
  install     install macOS
  completion  generate shell completion
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# How to build
```sh
./build/build.sh
```

# Install shell completion
```sh
# fish
vz completion | tee ~/.config/fish/completions/vz.fish
# zsh
vz completion | sudo tee /usr/local/share/zsh/site-functions/_vz
```

# Notes
* refer to swift version, https://github.com/neowu/vz-swift
* use `vz ls` to find ip, or check `cat /var/db/dhcpd_leases`
* for local docker host, refer to [setup-docker-host.md](doc/setup-docker-host.md)

# Known issues
* after macos updating, dhcp could broken due to firewall, either restart again, or manually unblock
```
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/libexec/bootpd
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /usr/libexec/bootpd
```
