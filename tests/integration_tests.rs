use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a mock sysfs structure for testing
fn create_mock_sysfs(base: &Path) {
    // Create mock hwmon
    let hwmon = base.join("sys/class/hwmon/hwmon0");
    fs::create_dir_all(&hwmon).unwrap();
    fs::write(hwmon.join("name"), "coretemp\n").unwrap();
    fs::write(hwmon.join("temp1_input"), "45000\n").unwrap();
    fs::write(hwmon.join("temp1_label"), "Core 0\n").unwrap();

    // Create mock thermal zone
    let thermal = base.join("sys/class/thermal/thermal_zone0");
    fs::create_dir_all(&thermal).unwrap();
    fs::write(thermal.join("type"), "x86_pkg_temp\n").unwrap();
    fs::write(thermal.join("temp"), "50000\n").unwrap();
    fs::write(thermal.join("trip_point_0_temp"), "100000\n").unwrap();
    fs::write(thermal.join("trip_point_0_type"), "critical\n").unwrap();

    // Create mock nvme device
    let nvme = base.join("sys/class/nvme/nvme0");
    fs::create_dir_all(&nvme).unwrap();
    fs::write(nvme.join("model"), "Samsung SSD 980 PRO\n").unwrap();
    fs::write(nvme.join("serial"), "S5GXNF0N123456\n").unwrap();
    fs::write(nvme.join("firmware_rev"), "5B2QGXA7\n").unwrap();
    fs::write(nvme.join("state"), "live\n").unwrap();

    // Create mock power supply (battery)
    let battery = base.join("sys/class/power_supply/BAT0");
    fs::create_dir_all(&battery).unwrap();
    fs::write(battery.join("type"), "Battery\n").unwrap();
    fs::write(battery.join("status"), "Discharging\n").unwrap();
    fs::write(battery.join("capacity"), "85\n").unwrap();
    fs::write(battery.join("voltage_now"), "12500000\n").unwrap();

    // Create mock RAPL
    let rapl = base.join("sys/class/powercap/intel-rapl:0");
    fs::create_dir_all(&rapl).unwrap();
    fs::write(rapl.join("name"), "package-0\n").unwrap();
    fs::write(rapl.join("energy_uj"), "123456789\n").unwrap();
    fs::write(rapl.join("max_energy_range_uj"), "262143328850\n").unwrap();
}

#[test]
fn test_mock_sysfs_structure_is_valid() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    // Verify structure exists
    assert!(dir.path().join("sys/class/hwmon/hwmon0/name").exists());
    assert!(
        dir.path()
            .join("sys/class/thermal/thermal_zone0/temp")
            .exists()
    );
    assert!(dir.path().join("sys/class/nvme/nvme0/model").exists());
    assert!(
        dir.path()
            .join("sys/class/power_supply/BAT0/capacity")
            .exists()
    );
    assert!(
        dir.path()
            .join("sys/class/powercap/intel-rapl:0/energy_uj")
            .exists()
    );
}

#[test]
fn test_hwmon_file_content() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    let content =
        fs::read_to_string(dir.path().join("sys/class/hwmon/hwmon0/temp1_input")).unwrap();
    assert_eq!(content.trim(), "45000");
}

#[test]
fn test_thermal_zone_file_content() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    let temp = fs::read_to_string(dir.path().join("sys/class/thermal/thermal_zone0/temp")).unwrap();
    assert_eq!(temp.trim(), "50000");

    let trip_type = fs::read_to_string(
        dir.path()
            .join("sys/class/thermal/thermal_zone0/trip_point_0_type"),
    )
    .unwrap();
    assert_eq!(trip_type.trim(), "critical");
}

#[test]
fn test_nvme_file_content() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    let model = fs::read_to_string(dir.path().join("sys/class/nvme/nvme0/model")).unwrap();
    assert_eq!(model.trim(), "Samsung SSD 980 PRO");

    let state = fs::read_to_string(dir.path().join("sys/class/nvme/nvme0/state")).unwrap();
    assert_eq!(state.trim(), "live");
}

#[test]
fn test_power_supply_file_content() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    let capacity =
        fs::read_to_string(dir.path().join("sys/class/power_supply/BAT0/capacity")).unwrap();
    assert_eq!(capacity.trim(), "85");

    let status = fs::read_to_string(dir.path().join("sys/class/power_supply/BAT0/status")).unwrap();
    assert_eq!(status.trim(), "Discharging");
}

#[test]
fn test_rapl_file_content() {
    let dir = TempDir::new().unwrap();
    create_mock_sysfs(dir.path());

    let energy =
        fs::read_to_string(dir.path().join("sys/class/powercap/intel-rapl:0/energy_uj")).unwrap();
    assert_eq!(energy.trim(), "123456789");

    let name = fs::read_to_string(dir.path().join("sys/class/powercap/intel-rapl:0/name")).unwrap();
    assert_eq!(name.trim(), "package-0");
}
