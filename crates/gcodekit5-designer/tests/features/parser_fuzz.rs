//! Property-based fuzz tests for parsers and G-code generation.
//!
//! Uses proptest to feed arbitrary strings into parsers to ensure they
//! never panic on malformed input.

use gcodekit5_core::Units;
use gcodekit5_designer::dxf_parser::DxfParser;
use gcodekit5_designer::gcode_gen::ToolpathToGcode;
use gcodekit5_designer::import::SvgImporter;
use gcodekit5_designer::model::Point;
use gcodekit5_designer::toolpath::{Toolpath, ToolpathSegment, ToolpathSegmentType};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// DXF Parser fuzz
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn dxf_parser_never_panics_on_arbitrary_input(input in "\\PC*") {
        let _ = DxfParser::parse(&input);
    }

    #[test]
    fn dxf_parser_handles_section_like_strings(
        section in "(HEADER|ENTITIES|BLOCKS|TABLES|EOF|SECTION|ENDSEC)",
        noise in "[0-9A-Z ]{0,40}"
    ) {
        let content = format!("0\n{}\n2\n{}\n0\nEOF\n", section, noise);
        let _ = DxfParser::parse(&content);
    }

    #[test]
    fn dxf_parser_handles_entity_fragments(
        entity in "(LINE|CIRCLE|ARC|POLYLINE|LWPOLYLINE|TEXT|POINT|SPLINE)",
        x in -1e6f64..1e6,
        y in -1e6f64..1e6,
    ) {
        let content = format!(
            "0\nSECTION\n2\nENTITIES\n0\n{}\n10\n{}\n20\n{}\n0\nENDSEC\n0\nEOF\n",
            entity, x, y
        );
        let _ = DxfParser::parse(&content);
    }

    #[test]
    fn dxf_parser_handles_extreme_values(
        val in prop::num::f64::ANY,
    ) {
        let content = format!(
            "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n0\nENDSEC\n0\nEOF\n",
            val, val, val, val
        );
        let _ = DxfParser::parse(&content);
    }
}

// ---------------------------------------------------------------------------
// SVG Importer fuzz
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn svg_importer_never_panics_on_arbitrary_input(input in "\\PC*") {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let _ = importer.import_string(&input);
    }

    #[test]
    fn svg_importer_handles_svg_fragments(
        tag in "(rect|circle|line|path|ellipse|polygon|polyline)",
        x in -1000.0f64..1000.0,
        y in -1000.0f64..1000.0,
    ) {
        let content = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><{} x="{}" y="{}"/></svg>"#,
            tag, x, y
        );
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let _ = importer.import_string(&content);
    }
}

// ---------------------------------------------------------------------------
// G-code generation fuzz
// ---------------------------------------------------------------------------

fn arb_point() -> impl Strategy<Value = Point> {
    (-1e4f64..1e4, -1e4f64..1e4).prop_map(|(x, y)| Point::new(x, y))
}

fn arb_segment_type() -> impl Strategy<Value = ToolpathSegmentType> {
    prop_oneof![
        Just(ToolpathSegmentType::RapidMove),
        Just(ToolpathSegmentType::LinearMove),
        Just(ToolpathSegmentType::ArcCW),
        Just(ToolpathSegmentType::ArcCCW),
    ]
}

fn arb_segment() -> impl Strategy<Value = ToolpathSegment> {
    (
        arb_segment_type(),
        arb_point(),
        arb_point(),
        arb_point(),
        100.0f64..5000.0,
        500u32..30000,
        proptest::option::of(-50.0f64..0.0),
    )
        .prop_map(|(seg_type, start, end, center, feed, spindle, z_depth)| {
            let mut seg = if matches!(
                seg_type,
                ToolpathSegmentType::ArcCW | ToolpathSegmentType::ArcCCW
            ) {
                ToolpathSegment::new_arc(seg_type, start, end, center, feed, spindle)
            } else {
                ToolpathSegment::new(seg_type, start, end, feed, spindle)
            };
            if let Some(z) = z_depth {
                seg = seg.with_z_depth(z);
            }
            seg
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn gcode_gen_never_panics_on_random_toolpath(
        segments in prop::collection::vec(arb_segment(), 1..20),
        tool_diameter in 0.1f64..50.0,
        depth in -50.0f64..-0.1,
        safe_z in 1.0f64..100.0,
        line_numbers in proptest::bool::ANY,
        num_axes in 2u8..4,
    ) {
        let mut toolpath = Toolpath::new(tool_diameter, depth);
        for seg in segments {
            toolpath.add_segment(seg);
        }
        let gen = ToolpathToGcode::with_line_numbers(Units::MM, safe_z, line_numbers);
        let mut gen = gen;
        gen.num_axes = num_axes;
        let output = gen.generate(&toolpath);
        // Output should be non-empty and valid UTF-8 (it is a String)
        prop_assert!(!output.is_empty());
        // Should contain header comment
        prop_assert!(output.contains("; Generated G-code"));
        // Should contain footer
        prop_assert!(output.contains("M30"));
    }

    #[test]
    fn gcode_gen_body_never_panics(
        segments in prop::collection::vec(arb_segment(), 0..30),
        tool_diameter in 0.1f64..50.0,
        depth in -50.0f64..-0.1,
        safe_z in 1.0f64..100.0,
    ) {
        let mut toolpath = Toolpath::new(tool_diameter, depth);
        for seg in segments {
            toolpath.add_segment(seg);
        }
        let gen = ToolpathToGcode::new(Units::MM, safe_z);
        let body = gen.generate_body(&toolpath, 10);
        // Body is a valid string (no panic)
        let _ = body.len();
    }
}

// ---------------------------------------------------------------------------
// Serialization round-trip fuzz
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn json_deserialization_never_panics(input in "\\PC{0,500}") {
        // DesignFile uses serde_json, so feed arbitrary strings
        let _ = serde_json::from_str::<serde_json::Value>(&input);
    }
}
