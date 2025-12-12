use crate::editor::EditorBridge;
use crate::ui::gtk::device_info::CapabilityItem;
use crate::ui::main_window::MainWindow;
use gcodekit5_communication::firmware::firmware_version::{FirmwareType, SemanticVersion};
use gcodekit5_communication::{list_ports, CapabilityManager};

/// Copy text to clipboard using arboard crate
pub fn copy_to_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(text.to_string()) {
            Ok(_) => {
                // Keep clipboard alive for a moment to ensure managers see it
                std::thread::sleep(std::time::Duration::from_millis(100));
                true
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Get list of available serial ports as friendly strings
pub fn get_available_ports() -> anyhow::Result<Vec<String>> {
    match list_ports() {
        Ok(ports) => {
            let port_names: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();
            if port_names.is_empty() {
                Ok(vec!["No ports available".to_string()])
            } else {
                Ok(port_names)
            }
        }
        Err(_) => Ok(vec!["Error reading ports".to_string()]),
    }
}

// ConfigSetting type is internal to the app root; for UI we define a local compatible version
#[derive(Debug, Clone)]
pub struct ConfigSettingRow {
    pub number: i32,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub description: String,
    pub category: String,
    pub read_only: bool,
}

/// Parse a GRBL setting line; adapted for UI view model use
pub fn parse_grbl_setting_line(line: &str) -> Option<ConfigSettingRow> {
    let line = line.trim();
    if !line.starts_with('$') {
        return None;
    }

    let line = &line[1..];
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() != 2 {
        return None;
    }

    let number = parts[0].parse::<i32>().ok()?;
    let value = parts[1].to_string();
    let (name, desc, unit, category) = get_grbl_setting_info(number);

    Some(ConfigSettingRow {
        number,
        name: name.to_string(),
        value: value.to_string(),
        unit: unit.to_string(),
        description: desc.to_string(),
        category: category.to_string(),
        read_only: false,
    })
}

pub fn get_grbl_setting_info(
    number: i32,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match number {
        0 => (
            "Step pulse time",
            "Step pulse duration in microseconds",
            "μs",
            "System",
        ),
        1 => (
            "Step idle delay",
            "Step idle delay in milliseconds",
            "ms",
            "System",
        ),
        2 => ("Step pulse invert", "Step pulse invert mask", "", "System"),
        3 => (
            "Step direction invert",
            "Step direction invert mask",
            "",
            "System",
        ),
        // ... (copy further entries as needed) ...
        132 => ("Z max travel", "Z-axis maximum travel", "mm", "Max Travel"),
        _ => (
            Box::leak(format!("${}", number).into_boxed_str()),
            "Unknown setting",
            "",
            "Other",
        ),
    }
}

/// Sync firmware capabilities into a `MainWindow` instance
pub fn sync_capabilities_to_ui(window: &MainWindow, capability_manager: &CapabilityManager) {
    let state = capability_manager.get_state();

    // Build capability list for the view
    let mut capabilities = Vec::new();

    capabilities.push(CapabilityItem {
        name: "Arc Support (G2/G3)".into(),
        enabled: state.supports_arcs,
        notes: "Circular interpolation commands".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Variable Spindle (M3/M4 S)".into(),
        enabled: state.supports_variable_spindle,
        notes: "PWM spindle speed control".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Probing (G38.x)".into(),
        enabled: state.supports_probing,
        notes: "Touch probe operations".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Tool Change (M6 T)".into(),
        enabled: state.supports_tool_change,
        notes: "Automatic tool changing".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Homing Cycle ($H)".into(),
        enabled: state.supports_homing,
        notes: "Machine homing to limit switches".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Feed/Spindle Overrides".into(),
        enabled: state.supports_overrides,
        notes: "Real-time adjustment of feed and spindle".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Laser Mode (M3/M4)".into(),
        enabled: state.supports_laser,
        notes: "Dynamic laser power control for engraving/cutting".into(),
    });
    capabilities.push(CapabilityItem {
        name: format!("{} Axes Support", state.max_axes).into(),
        enabled: state.max_axes > 0,
        notes: format!("Maximum {} axes (X,Y,Z,A,B,C)", state.max_axes).into(),
    });
    capabilities.push(CapabilityItem {
        name: format!("{} Coordinate Systems", state.coordinate_systems).into(),
        enabled: state.coordinate_systems > 0,
        notes: "Work coordinate systems (G54-G59)".into(),
    });

    // Update UI
    window.set_device_capabilities(capabilities);
}

/// Update device info panel on the passed MainWindow
pub fn update_device_info_panel(
    window: &MainWindow,
    firmware_type: FirmwareType,
    version: SemanticVersion,
    capability_manager: &CapabilityManager,
) {
    // Update capability manager with detected firmware
    capability_manager.update_firmware(firmware_type, version.clone());

    window.set_device_firmware_type(format!("{:?}", firmware_type));
    window.set_device_firmware_version(version.to_string());
    window.set_device_name(format!("{:?} Device", firmware_type));

    // Build capabilities vector
    let state = capability_manager.get_state();
    let mut capabilities = Vec::new();

    capabilities.push(CapabilityItem {
        name: "Arc Support (G2/G3)".into(),
        enabled: state.supports_arcs,
        notes: "Circular interpolation commands".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Variable Spindle (M3/M4 S)".into(),
        enabled: state.supports_variable_spindle,
        notes: "PWM spindle speed control".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Probing (G38.x)".into(),
        enabled: state.supports_probing,
        notes: "Touch probe operations".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Tool Change (M6 T)".into(),
        enabled: state.supports_tool_change,
        notes: "Automatic tool changing".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Homing Cycle ($H)".into(),
        enabled: state.supports_homing,
        notes: "Machine homing to limit switches".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Feed/Spindle Overrides".into(),
        enabled: state.supports_overrides,
        notes: "Real-time adjustment of feed and spindle".into(),
    });
    capabilities.push(CapabilityItem {
        name: "Laser Mode (M3/M4)".into(),
        enabled: state.supports_laser,
        notes: "Dynamic laser power control for engraving/cutting".into(),
    });

    capabilities.push(CapabilityItem {
        name: format!("{} Axes Support", state.max_axes).into(),
        enabled: state.max_axes > 0,
        notes: format!("Maximum {} axes (X,Y,Z,A,B,C)", state.max_axes).into(),
    });

    capabilities.push(CapabilityItem {
        name: format!("{} Coordinate Systems", state.coordinate_systems).into(),
        enabled: state.coordinate_systems > 0,
        notes: "Work coordinate systems (G54-G59)".into(),
    });

    window.set_device_capabilities(capabilities);
}

/// Update visible lines in the editor view
pub fn update_visible_lines(window: &MainWindow, editor_bridge: &EditorBridge) {
    let (start_line, end_line) = editor_bridge.viewport_range();
    let mut visible_lines = Vec::new();
    for i in start_line..end_line {
        if let Some(content) = editor_bridge.get_line_at(i) {
            visible_lines.push(crate::TextLine {
                line_number: (i + 1) as i32,
                content: content.clone(),
                is_dirty: false,
            });
        }
    }
    window.set_visible_lines(visible_lines);
}
