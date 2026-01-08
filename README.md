# rs-linux-exporter
Prometheus-compatible Linux metrics exporter written in Rust.

## Overview
rs-linux-exporter aims to provide the most important Linux host metrics with a
simple, safe, and fast Rust implementation. The goal is to avoid external
dependencies (especially tools and unsafe languages) to keep the binary small
and the software reliable.

## Goals
- Provide essential Linux metrics for Prometheus scraping
- Keep the binary size small and startup time fast
- Avoid external tool dependencies and unsafe language bindings
- Favor correctness, safety, and predictable performance

## Planned metrics (initial focus)
- System uptime
- CPU usage and load averages
- Memory usage (total/free/available/buffers/cache)
- Disk usage (total/used/free per filesystem)
- Disk I/O (bytes, ops, time per device)
- Network I/O (bytes, packets, errors per interface)
- Process count and basic system limits

## Kernel Modules for Hardware Monitoring

The `hwmon` and `thermal` exporters require appropriate kernel modules to be loaded.
Use `sensors-detect` from the `lm-sensors` package to identify which modules your
system needs.

### Common Modules

| Module | Description |
|--------|-------------|
| `coretemp` | Intel CPU temperature sensors |
| `k10temp` | AMD CPU temperature sensors (Family 10h+) |
| `nct6775` | Nuvoton Super I/O chips (common on many motherboards) |
| `it87` | ITE Super I/O chips |
| `drivetemp` | SATA/SAS drive temperatures (kernel 5.6+) |

### Quick Setup

```bash
# Detect and load modules interactively
sudo sensors-detect

# Or load common modules manually
sudo modprobe coretemp    # Intel CPUs
sudo modprobe k10temp     # AMD CPUs
sudo modprobe drivetemp   # Drive temperatures

# Make persistent at boot
echo -e "coretemp\nk10temp\ndrivetemp" | sudo tee /etc/modules-load.d/sensors.conf
```

### Troubleshooting

If sensors are missing:
- Re-run `sensors-detect` and follow its recommendations
- Check loaded modules: `lsmod | grep -E 'coretemp|k10temp|nct|it87|drivetemp'`
- Some motherboard chips (IT8655E, IT8625E, IT8686E) may need out-of-tree drivers
- For additional readings, try boot parameter: `acpi_enforce_resources=lax`

See [lm_sensors ArchWiki](https://wiki.archlinux.org/title/Lm_sensors) for detailed guidance.

## Configuration

Configuration is optional. Create a `config.toml` file in the working directory.

### Example config.toml

```toml
# Ignore loop devices in filesystem metrics
ignore_loop_devices = true

# Ignore PPP interfaces in network metrics
ignore_ppp_interfaces = true

# Ignore veth and br-* interfaces in network metrics
ignore_veth_interfaces = true

# Disable specific datasources (will not be polled)
# Available: procfs, cpufreq, softnet, conntrack, filesystems, hwmon, thermal
disabled_datasources = ["thermal", "conntrack"]
```

### Available Datasources

| Datasource | Description |
|------------|-------------|
| `procfs` | System stats from /proc (CPU, memory, network, disk I/O) |
| `cpufreq` | CPU frequency per core |
| `softnet` | Network soft interrupt statistics |
| `conntrack` | Connection tracking statistics |
| `filesystems` | Filesystem usage statistics |
| `hwmon` | Hardware sensors (temperature, fan, voltage, power) |
| `thermal` | Thermal zones and cooling devices |
| `rapl` | Intel/AMD RAPL energy consumption (CPU, DRAM) |
| `power_supply` | Battery and AC adapter status |
| `nvme` | NVMe device information (model, serial, state) |
| `edac` | Memory error detection (correctable/uncorrectable) |
| `numa` | NUMA node memory and hit/miss statistics |

## Contributing
Please check dependency freshness and binary size when submitting changes.

To install helper tools:
- `cargo install cargo-outdated`
- `cargo install cargo-bloat`

Recommended checks:
- `cargo outdated`
- `cargo bloat --release`

## Status
This project is a work in progress.
