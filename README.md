A lightweight virtual machine tool, for personal use.

only support Apple Silicon and macOS Sonoma+

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
  edit        edit vm (cpu, ram, increase disk image size)
  install     install macOS
  completion  generate shell completion
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Create
```
create vm

Usage: vz create [OPTIONS] <NAME>

Arguments:
  <NAME>  vm name

Options:
      --os <OS>      create a linux or macOS vm [default: linux] [possible values: linux, macOS]
      --cpu <CPU>    cpu count [default: 1]
      --ram <RAM>    ram size in gb [default: 1]
      --disk <DISK>  disk size in gb [default: 50]
      --ipsw <IPSW>  macOS restore image file, e.g. --ipsw=UniversalMac_14.5_23F79_Restore.ipsw
  -h, --help         Print help
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
* all data is stored at `~/.local/share/vz`
* use `vz ls` to find ip, or check `cat /var/db/dhcpd_leases`
* for local docker host, refer to [setup-docker-host.md](doc/setup-docker-host.md)
* refer to swift version if interested, https://github.com/neowu/vz-swift

# Known issues
* after macos updating, dhcp could broken due to firewall, either restart again, or manually unblock
```
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/libexec/bootpd
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /usr/libexec/bootpd
```
