// Test harness for communication protocols: GRBL response parsing,
// command creation, and protocol round-trips.

use gcodekit5_communication::firmware::grbl::{
    BufferState, CommandCreator, GrblResponse, GrblResponseParser, JogCommand, JogMode,
    ProbeCommand, ProbeType, RealTimeCommand, SystemCommand,
};
use gcodekit5_core::{CNCPoint, Units};

// ── GrblResponseParser Tests ──────────────────────────────────────────

#[test]
fn test_parse_ok() {
    let parser = GrblResponseParser::new();
    assert_eq!(parser.parse("ok"), Some(GrblResponse::Ok));
}

#[test]
fn test_parse_ok_with_whitespace() {
    let parser = GrblResponseParser::new();
    assert_eq!(parser.parse("  ok  "), Some(GrblResponse::Ok));
}

#[test]
fn test_parse_error_codes() {
    let parser = GrblResponseParser::new();
    for code in 1..=24 {
        let input = format!("error:{}", code);
        let result = parser.parse(&input);
        assert_eq!(
            result,
            Some(GrblResponse::Error(code)),
            "Failed to parse error code {}",
            code
        );
    }
}

#[test]
fn test_parse_alarm_codes() {
    let parser = GrblResponseParser::new();
    for code in 1..=9 {
        let input = format!("alarm:{}", code);
        let result = parser.parse(&input);
        assert_eq!(
            result,
            Some(GrblResponse::Alarm(code)),
            "Failed to parse alarm code {}",
            code
        );
    }
}

#[test]
fn test_parse_version() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("Grbl 1.1h ['$' for help]");
    assert!(matches!(result, Some(GrblResponse::Version(_))));
}

#[test]
fn test_parse_build_info() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("[VER:1.1h.20190825:]");
    assert!(matches!(result, Some(GrblResponse::BuildInfo(_))));
}

#[test]
fn test_parse_setting() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("$0=10");
    assert_eq!(
        result,
        Some(GrblResponse::Setting {
            number: 0,
            value: "10".to_string()
        })
    );
}

#[test]
fn test_parse_multiple_settings() {
    let parser = GrblResponseParser::new();
    let settings = vec![
        ("$0=10", 0, "10"),
        ("$1=25", 1, "25"),
        ("$2=0", 2, "0"),
        ("$10=1", 10, "1"),
        ("$100=250.000", 100, "250.000"),
    ];
    for (input, num, val) in settings {
        let result = parser.parse(input);
        assert_eq!(
            result,
            Some(GrblResponse::Setting {
                number: num,
                value: val.to_string()
            }),
            "Failed to parse: {}",
            input
        );
    }
}

#[test]
fn test_parse_empty_line() {
    let parser = GrblResponseParser::new();
    assert_eq!(parser.parse(""), None);
    assert_eq!(parser.parse("  "), None);
}

#[test]
fn test_parse_generic_message() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("some random text");
    assert!(matches!(result, Some(GrblResponse::Message(_))));
}

// ── Status Report Parsing ─────────────────────────────────────────────

#[test]
fn test_parse_status_idle() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Idle|MPos:0.000,0.000,0.000>");
    match result {
        Some(GrblResponse::Status(status)) => {
            assert_eq!(status.state, "Idle");
            assert!((status.machine_pos.x - 0.0).abs() < 1e-6);
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

#[test]
fn test_parse_status_run() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Run|MPos:12.500,-3.200,5.000|F:1200>");
    match result {
        Some(GrblResponse::Status(status)) => {
            assert_eq!(status.state, "Run");
            assert!((status.machine_pos.x - 12.5).abs() < 1e-6);
            assert!((status.machine_pos.y - (-3.2)).abs() < 1e-6);
            assert!((status.machine_pos.z - 5.0).abs() < 1e-6);
            assert_eq!(status.feed_rate, Some(1200.0));
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

#[test]
fn test_parse_status_with_buffer() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Idle|MPos:0.000,0.000,0.000|Buf:15:128>");
    match result {
        Some(GrblResponse::Status(status)) => {
            assert_eq!(
                status.buffer_state,
                Some(BufferState {
                    plan: 15,
                    exec: 128
                })
            );
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

#[test]
fn test_parse_status_with_spindle() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Run|MPos:0.000,0.000,0.000|S:10000>");
    match result {
        Some(GrblResponse::Status(status)) => {
            assert_eq!(status.spindle_speed, Some(10000));
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

#[test]
fn test_parse_status_with_wco() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Idle|MPos:0.000,0.000,0.000|WCO:10.000,20.000,5.000>");
    match result {
        Some(GrblResponse::Status(status)) => {
            let wco = status.work_coord_offset.unwrap();
            assert!((wco.x - 10.0).abs() < 1e-6);
            assert!((wco.y - 20.0).abs() < 1e-6);
            assert!((wco.z - 5.0).abs() < 1e-6);
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

#[test]
fn test_parse_status_various_states() {
    let parser = GrblResponseParser::new();
    for state in &[
        "Idle", "Run", "Hold", "Door", "Home", "Alarm", "Check", "Sleep",
    ] {
        let input = format!("<{}|MPos:0.000,0.000,0.000>", state);
        match parser.parse(&input) {
            Some(GrblResponse::Status(status)) => {
                assert_eq!(status.state, *state);
            }
            other => panic!("Expected Status for state '{}', got {:?}", state, other),
        }
    }
}

#[test]
fn test_parse_status_with_work_pos() {
    let parser = GrblResponseParser::new();
    let result = parser.parse("<Idle|WPos:1.500,2.500,3.500>");
    match result {
        Some(GrblResponse::Status(status)) => {
            assert!((status.work_pos.x - 1.5).abs() < 1e-6);
            assert!((status.work_pos.y - 2.5).abs() < 1e-6);
            assert!((status.work_pos.z - 3.5).abs() < 1e-6);
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

// ── Error & Alarm Descriptions ────────────────────────────────────────

#[test]
fn test_error_descriptions() {
    assert_eq!(
        GrblResponseParser::error_description(1),
        "Expected command letter"
    );
    assert_eq!(
        GrblResponseParser::error_description(20),
        "Unsupported or invalid g-code command"
    );
    assert_eq!(GrblResponseParser::error_description(255), "Unknown error");
}

#[test]
fn test_alarm_descriptions() {
    assert_eq!(
        GrblResponseParser::alarm_description(1),
        "Hard limit triggered"
    );
    assert_eq!(GrblResponseParser::alarm_description(6), "Homing fail");
    assert_eq!(GrblResponseParser::alarm_description(255), "Unknown alarm");
}

// ── CommandCreator Tests ──────────────────────────────────────────────

#[test]
fn test_realtime_command_bytes() {
    assert_eq!(
        CommandCreator::real_time_command(RealTimeCommand::QueryStatus),
        vec![b'?']
    );
    assert_eq!(
        CommandCreator::real_time_command(RealTimeCommand::FeedHold),
        vec![b'!']
    );
    assert_eq!(
        CommandCreator::real_time_command(RealTimeCommand::CycleStart),
        vec![b'~']
    );
    assert_eq!(
        CommandCreator::real_time_command(RealTimeCommand::SoftReset),
        vec![0x18]
    );
}

#[test]
fn test_system_commands() {
    assert_eq!(
        CommandCreator::system_command(SystemCommand::HomeAll),
        "$H\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::KillAlarmLock),
        "$X\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::CheckMode),
        "$C\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::QueryParserState),
        "$G\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::QueryBuildInfo),
        "$I\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::ResetEeprom),
        "$RST=$\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::ResetAll),
        "$RST=*\n"
    );
    assert_eq!(
        CommandCreator::system_command(SystemCommand::Sleep),
        "$SLP\n"
    );
}

#[test]
fn test_convenience_commands() {
    assert_eq!(CommandCreator::soft_reset(), vec![0x18]);
    assert_eq!(CommandCreator::query_status(), vec![b'?']);
    assert_eq!(CommandCreator::feed_hold(), vec![b'!']);
    assert_eq!(CommandCreator::cycle_start(), vec![b'~']);
    assert_eq!(CommandCreator::home_all(), "$H\n");
    assert_eq!(CommandCreator::kill_alarm_lock(), "$X\n");
}

#[test]
fn test_spindle_commands() {
    assert_eq!(CommandCreator::spindle_on(10000), "M3 S10000\n");
    assert_eq!(CommandCreator::spindle_off(), "M5\n");
}

#[test]
fn test_coolant_commands() {
    assert_eq!(CommandCreator::coolant_on(), "M8\n");
    assert_eq!(CommandCreator::coolant_off(), "M9\n");
}

#[test]
fn test_program_commands() {
    assert_eq!(CommandCreator::program_pause(), "M0\n");
    assert_eq!(CommandCreator::program_end(), "M2\n");
}

#[test]
fn test_tool_change() {
    assert_eq!(CommandCreator::tool_change(1), "T1 M6\n");
    assert_eq!(CommandCreator::tool_change(5), "T5 M6\n");
}

#[test]
fn test_dwell() {
    assert_eq!(CommandCreator::dwell(1.5), "G4 P1.5\n");
    assert_eq!(CommandCreator::dwell(0.0), "G4 P0.0\n");
}

#[test]
fn test_rapid_move() {
    let cmd = CommandCreator::rapid_move(Some(10.0), Some(20.0), None);
    assert!(cmd.starts_with("G0"));
    assert!(cmd.contains("X10.000"));
    assert!(cmd.contains("Y20.000"));
    assert!(!cmd.contains("Z"));
}

#[test]
fn test_rapid_move_with_z() {
    let cmd = CommandCreator::rapid_move(None, None, Some(5.0));
    assert!(cmd.contains("Z5.000"));
    assert!(!cmd.contains("X"));
}

#[test]
fn test_linear_move() {
    let cmd = CommandCreator::linear_move(Some(10.0), Some(20.0), Some(-3.0), 150.0);
    assert!(cmd.starts_with("G1"));
    assert!(cmd.contains("X10.000"));
    assert!(cmd.contains("Y20.000"));
    assert!(cmd.contains("Z-3.000"));
    assert!(cmd.contains("F150"));
}

// ── JogCommand Tests ──────────────────────────────────────────────────

#[test]
fn test_jog_xy() {
    let target = CNCPoint::with_axes(10.0, 20.0, 0.0, 0.0, 0.0, 0.0, Units::MM);
    let jog = JogCommand::new(JogMode::XY, target, 500.0);
    let gcode = jog.to_gcode();
    assert!(gcode.starts_with("$J=G91 G0"));
    assert!(gcode.contains("X10.000"));
    assert!(gcode.contains("Y20.000"));
    assert!(gcode.contains("F500"));
}

#[test]
fn test_jog_xz() {
    let target = CNCPoint::with_axes(15.0, 0.0, -5.0, 0.0, 0.0, 0.0, Units::MM);
    let jog = JogCommand::new(JogMode::XZ, target, 300.0);
    let gcode = jog.to_gcode();
    assert!(gcode.starts_with("$J=G91 G0"));
    assert!(gcode.contains("X15.000"));
    assert!(gcode.contains("Z-5.000"));
}

#[test]
fn test_jog_yz() {
    let target = CNCPoint::with_axes(0.0, 25.0, -10.0, 0.0, 0.0, 0.0, Units::MM);
    let jog = JogCommand::new(JogMode::YZ, target, 200.0);
    let gcode = jog.to_gcode();
    assert!(gcode.starts_with("$J=G91 G0"));
    assert!(gcode.contains("Y25.000"));
    assert!(gcode.contains("Z-10.000"));
}

#[test]
fn test_jog_incremental() {
    let cmd = CommandCreator::jog_incremental("X", 5.0, 600.0);
    assert!(cmd.starts_with("$J=G91 G0"));
    assert!(cmd.contains("X"));
    assert!(cmd.contains("F600"));
}

// ── ProbeCommand Tests ────────────────────────────────────────────────

#[test]
fn test_probe_types() {
    assert_eq!(ProbeType::Touching.gcode_command(), "G38.2");
    assert_eq!(ProbeType::TouchingRequired.gcode_command(), "G38.3");
    assert_eq!(ProbeType::Backing.gcode_command(), "G38.4");
    assert_eq!(ProbeType::BackingRequired.gcode_command(), "G38.5");
}

#[test]
fn test_probe_command_gcode() {
    let target = CNCPoint::with_axes(0.0, 0.0, -10.0, 0.0, 0.0, 0.0, Units::MM);
    let probe = ProbeCommand::new(ProbeType::Touching, target, 50.0);
    let gcode = probe.to_gcode();
    assert!(gcode.starts_with("G38.2"));
    assert!(gcode.contains("Z-10.000"));
    assert!(gcode.contains("F50"));
}

#[test]
fn test_probe_command_creation() {
    let target = CNCPoint::with_axes(0.0, 0.0, -20.0, 0.0, 0.0, 0.0, Units::MM);
    let gcode = CommandCreator::probe(ProbeType::TouchingRequired, target, 100.0);
    assert!(gcode.starts_with("G38.3"));
}

// ── Work Offset ───────────────────────────────────────────────────────

#[test]
fn test_set_work_offset() {
    let cmd = CommandCreator::set_work_offset(&["X", "Y", "Z"]);
    assert!(cmd.starts_with("G10 P0"));
    assert!(cmd.contains("X0"));
    assert!(cmd.contains("Y0"));
    assert!(cmd.contains("Z0"));
}

// ── Response round-trip: parse what commands produce ───────────────────

#[test]
fn test_parse_round_trip_ok_response() {
    let parser = GrblResponseParser::new();
    // After sending a command, GRBL responds with "ok"
    let response = parser.parse("ok");
    assert_eq!(response, Some(GrblResponse::Ok));
}

#[test]
fn test_parse_round_trip_error_response() {
    let parser = GrblResponseParser::new();
    // After sending invalid command, GRBL responds with error
    let response = parser.parse("error:20");
    assert_eq!(response, Some(GrblResponse::Error(20)));
}

#[test]
fn test_parse_round_trip_status_query() {
    let parser = GrblResponseParser::new();
    // After sending '?', GRBL responds with status report
    let _ = CommandCreator::query_status(); // This is what we send
    let response = parser.parse("<Idle|MPos:10.000,20.000,5.000|F:100|S:3000>");
    match response {
        Some(GrblResponse::Status(status)) => {
            assert_eq!(status.state, "Idle");
            assert!((status.machine_pos.x - 10.0).abs() < 1e-6);
            assert!((status.machine_pos.y - 20.0).abs() < 1e-6);
            assert!((status.machine_pos.z - 5.0).abs() < 1e-6);
            assert_eq!(status.feed_rate, Some(100.0));
            assert_eq!(status.spindle_speed, Some(3000));
        }
        other => panic!("Expected Status, got {:?}", other),
    }
}

// ── RealTimeCommand properties ────────────────────────────────────────

#[test]
fn test_realtime_descriptions() {
    assert_eq!(RealTimeCommand::QueryStatus.description(), "Query Status");
    assert_eq!(RealTimeCommand::FeedHold.description(), "Feed Hold");
    assert_eq!(RealTimeCommand::CycleStart.description(), "Cycle Start");
    assert_eq!(RealTimeCommand::SoftReset.description(), "Soft Reset");
}

#[test]
fn test_system_command_descriptions() {
    assert_eq!(SystemCommand::HomeAll.description(), "Home All Axes");
    assert_eq!(SystemCommand::CheckMode.description(), "Check Mode");
    assert_eq!(SystemCommand::Sleep.description(), "Sleep");
}

// ── GrblResponse Display ──────────────────────────────────────────────

#[test]
fn test_response_display() {
    assert_eq!(format!("{}", GrblResponse::Ok), "ok");
    let setting = GrblResponse::Setting {
        number: 10,
        value: "1".to_string(),
    };
    assert_eq!(format!("{}", setting), "setting:$10=1");
}

// ── Probe descriptions ───────────────────────────────────────────────

#[test]
fn test_probe_descriptions() {
    assert_eq!(ProbeType::Touching.description(), "Probe to Contact");
    assert_eq!(ProbeType::Backing.description(), "Probe Away");
}
