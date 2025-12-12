use gcodekit5_designer::{shapes::TextShape, ToolpathGenerator};

#[test]
fn test_text_toolpath_advances_characters() {
    let gen = ToolpathGenerator::new();

    let text = TextShape {
        text: "AB".to_string(),
        x: 0.0,
        y: 0.0,
        font_size: 10.0,
        rotation: 0.0,
        font_family: "Sans".to_string(),
        bold: false,
        italic: false,
    };

    let toolpaths = gen.generate_text_toolpath(&text, 1.0);

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;

    for tp in toolpaths {
        for seg in tp.segments {
            min_x = min_x.min(seg.start.x).min(seg.end.x);
            max_x = max_x.max(seg.start.x).max(seg.end.x);
        }
    }

    // Two glyphs should span more than a couple of mm at 10mm font size.
    assert!(max_x - min_x > 5.0);
}

#[test]
fn test_text_toolpath_contains_many_segments_for_curves() {
    let gen = ToolpathGenerator::new();

    let text = TextShape {
        text: "S".to_string(),
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        rotation: 0.0,
        font_family: "Sans".to_string(),
        bold: false,
        italic: false,
    };

    let toolpaths = gen.generate_text_toolpath(&text, 1.0);
    let seg_count: usize = toolpaths.iter().map(|tp| tp.segments.len()).sum();

    // Curvy glyphs should not collapse down to a handful of straight segments.
    assert!(seg_count > 25);
}
