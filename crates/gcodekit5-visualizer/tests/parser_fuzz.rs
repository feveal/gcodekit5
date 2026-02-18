//! Property-based fuzz tests for the G-code parser.
//!
//! Uses proptest to feed arbitrary strings into the GcodeParser to ensure
//! it never panics on malformed input.

use gcodekit5_visualizer::GcodeParser;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn gcode_parser_never_panics_on_arbitrary_input(input in "\\PC*") {
        let mut parser = GcodeParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn gcode_parser_handles_gcode_fragments(
        code in "(G00|G01|G02|G03|G17|G18|G19|G20|G21|G90|G91|M3|M5|M30)",
        x in -1e4f64..1e4,
        y in -1e4f64..1e4,
        z in -1e4f64..1e4,
    ) {
        let input = format!("{} X{:.3} Y{:.3} Z{:.3}", code, x, y, z);
        let mut parser = GcodeParser::new();
        let result = parser.parse(&input);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn gcode_parser_handles_comments(
        comment_style in "(;|\\()",
        text in "[A-Za-z0-9 ]{0,40}"
    ) {
        let input = format!("G01 X10 Y20 {}{}", comment_style, text);
        let mut parser = GcodeParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn gcode_parser_preserves_modal_state(
        lines in prop::collection::vec(
            "(G00|G01|G90|G91|G20|G21) X[0-9]{1,3} Y[0-9]{1,3}",
            1..20
        )
    ) {
        let mut parser = GcodeParser::new();
        for line in &lines {
            let _ = parser.parse(line);
        }
        // State should be consistent - no crash
        let _state = parser.get_state();
    }

    #[test]
    fn gcode_parser_handles_line_numbers(
        n in 0u32..99999,
        code in "(G00|G01|G02|G03)",
        x in -1e3f64..1e3,
    ) {
        let input = format!("N{} {} X{:.3}", n, code, x);
        let mut parser = GcodeParser::new();
        let _ = parser.parse(&input);
    }

    #[test]
    fn gcode_parser_handles_extreme_values(
        x in prop::num::f64::ANY,
        y in prop::num::f64::ANY,
    ) {
        let input = format!("G01 X{} Y{}", x, y);
        let mut parser = GcodeParser::new();
        let _ = parser.parse(&input);
    }
}
