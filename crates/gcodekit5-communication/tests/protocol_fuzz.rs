//! Property-based fuzz tests for communication protocol parsers.
//!
//! Uses proptest to feed arbitrary strings into parsers to ensure they
//! never panic on malformed input.

use gcodekit5_communication::firmware::grbl::response_parser::GrblResponseParser;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// GRBL Response Parser fuzz
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn grbl_parser_never_panics_on_arbitrary_input(input in "\\PC*") {
        let parser = GrblResponseParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn grbl_parser_handles_status_like_strings(
        state in "(Idle|Run|Hold|Alarm|Check|Home|Sleep|Jog|Door)",
        x in -1e4f64..1e4,
        y in -1e4f64..1e4,
        z in -1e4f64..1e4,
    ) {
        let input = format!("<{}|MPos:{:.3},{:.3},{:.3}>", state, x, y, z);
        let parser = GrblResponseParser::new();
        let result = parser.parse(&input);
        // Valid status lines should parse successfully
        prop_assert!(result.is_some());
    }

    #[test]
    fn grbl_parser_handles_error_codes(code in 0u8..255) {
        let input = format!("error:{}", code);
        let parser = GrblResponseParser::new();
        let result = parser.parse(&input);
        prop_assert!(result.is_some());
    }

    #[test]
    fn grbl_parser_handles_alarm_codes(code in 0u8..255) {
        let input = format!("ALARM:{}", code);
        let parser = GrblResponseParser::new();
        let result = parser.parse(&input);
        prop_assert!(result.is_some());
    }

    #[test]
    fn grbl_parser_handles_setting_lines(
        num in 0u8..255,
        val in "[0-9.]{1,10}"
    ) {
        let input = format!("${}={}", num, val);
        let parser = GrblResponseParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn grbl_parser_handles_malformed_status(
        inner in "[A-Za-z0-9|:,. ]{0,100}"
    ) {
        let input = format!("<{}>", inner);
        let parser = GrblResponseParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn grbl_parser_handles_partial_messages(
        prefix in "(ok|error|ALARM|\\$|<|\\[)",
        suffix in "\\PC{0,50}"
    ) {
        let input = format!("{}{}", prefix, suffix);
        let parser = GrblResponseParser::new();
        let _ = parser.parse(&input);
    }
}
