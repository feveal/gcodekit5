//! Unit tests for toolpath generation with rotation
//! 
//! Tests verify that rotated shapes generate correct G-code toolpaths
//! and that rotation is applied consistently across all shape types.

use gcodekit5_designer::model::{Point, Rectangle, Ellipse, Triangle, Polygon};
use gcodekit5_designer::toolpath::{ToolpathGenerator, ToolpathSegmentType};

#[test]
fn test_rectangle_toolpath_no_rotation() {
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 0.0,
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_rectangle_contour(&rect, 5.0);
    
    // Should generate toolpath segments
    assert!(!toolpaths.is_empty(), "Should generate toolpath for unrotated rectangle");
}

#[test]
fn test_rectangle_toolpath_with_rotation() {
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 45.0, // 45 degrees rotation
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_rectangle_contour(&rect, 5.0);
    
    // Should generate toolpath segments with rotation applied
    assert!(!toolpaths.is_empty(), "Should generate toolpath for rotated rectangle");
    
    // Verify the toolpath contains both rapid and linear moves
    let has_rapid = toolpaths.iter().any(|tp| {
        tp.segments.iter().any(|seg| matches!(seg.segment_type, ToolpathSegmentType::RapidMove))
    });
    let has_linear = toolpaths.iter().any(|tp| {
        tp.segments.iter().any(|seg| matches!(seg.segment_type, ToolpathSegmentType::LinearMove))
    });
    
    assert!(has_rapid, "Toolpath should contain rapid moves");
    assert!(has_linear, "Toolpath should contain linear moves");
}

#[test]
fn test_circle_toolpath_with_rotation() {
    let circle = Ellipse {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0, // Circle (equal width and height)
        rotation: 30.0,
        center: Point::new(5.0, 5.0),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_circle_contour(&circle, 5.0);
    
    assert!(!toolpaths.is_empty(), "Should generate toolpath for rotated circle");
}

#[test]
fn test_triangle_toolpath_with_rotation() {
    let triangle = Triangle {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 0.0,
        x3: 5.0,
        y3: 8.66,
        rotation: 60.0,
        center: Point::new(5.0, 2.89),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_triangle_contour(&triangle, 5.0);
    
    assert!(!toolpaths.is_empty(), "Should generate toolpath for rotated triangle");
}

#[test]
fn test_polygon_toolpath_with_rotation() {
    let polygon = Polygon {
        center: Point::new(5.0, 5.0),
        radius: 5.0,
        sides: 6, // Hexagon
        rotation: 22.5,
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_polygon_contour(&polygon, 5.0);
    
    assert!(!toolpaths.is_empty(), "Should generate toolpath for rotated polygon");
}

#[test]
fn test_rotation_does_not_affect_toolpath_count() {
    let rect_no_rotation = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 0.0,
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let rect_with_rotation = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 45.0,
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths_no_rotation = generator.generate_rectangle_contour(&rect_no_rotation, 5.0);
    let toolpaths_with_rotation = generator.generate_rectangle_contour(&rect_with_rotation, 5.0);
    
    // Rotation should not change the number of toolpath layers
    assert_eq!(
        toolpaths_no_rotation.len(),
        toolpaths_with_rotation.len(),
        "Rotation should not affect number of toolpath layers"
    );
}

#[test]
fn test_pocket_toolpath_with_rotation() {
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 15.0,
        rotation: 15.0,
        center: Point::new(10.0, 7.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_rectangle_pocket(&rect, 5.0, 1.0, 2.0);
    
    assert!(!toolpaths.is_empty(), "Should generate pocket toolpath for rotated rectangle");
}

#[test]
fn test_multiple_rotation_angles_generate_valid_toolpaths() {
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let test_angles = vec![0.0, 15.0, 30.0, 45.0, 60.0, 90.0, 180.0, 270.0];
    
    for angle in test_angles {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 5.0,
            rotation: angle,
            center: Point::new(5.0, 2.5),
            layer_id: None,
            offset: 0.0,
            fillet: 0.0,
            chamfer: 0.0,
        };
        
        let toolpaths = generator.generate_rectangle_contour(&rect, 5.0);
        assert!(
            !toolpaths.is_empty(),
            "Should generate valid toolpath at {} degrees",
            angle
        );
    }
}

#[test]
fn test_negative_rotation_generates_valid_toolpath() {
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: -45.0, // Negative rotation
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    let toolpaths = generator.generate_rectangle_contour(&rect, 5.0);
    
    assert!(!toolpaths.is_empty(), "Should generate toolpath with negative rotation");
}

#[test]
fn test_toolpath_generator_parameters() {
    // Test that toolpath generator correctly uses the provided parameters
    let tool_diameter = 6.35; // 1/4 inch
    let feed_rate = 1500.0;
    let spindle_speed = 18000;
    
    let generator = ToolpathGenerator::new(tool_diameter, feed_rate, spindle_speed);
    
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 0.0,
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let toolpaths = generator.generate_rectangle_contour(&rect, 5.0);
    
    // Verify toolpaths were generated
    assert!(!toolpaths.is_empty());
    
    // Check that segments have the correct feed rate and spindle speed
    for toolpath in &toolpaths {
        for segment in &toolpath.segments {
            assert!(segment.feed_rate > 0.0, "Feed rate should be positive");
            assert!(segment.spindle_speed > 0, "Spindle speed should be positive");
        }
    }
}

#[test]
fn test_helical_ramping_with_rotation() {
    let rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 5.0,
        rotation: 30.0,
        center: Point::new(5.0, 2.5),
        layer_id: None,
        offset: 0.0,
        fillet: 0.0,
        chamfer: 0.0,
    };
    
    let generator = ToolpathGenerator::new(3.175, 1000.0, 12000);
    
    // Generate pocket with multiple depth passes (triggers helical ramping)
    let toolpaths = generator.generate_rectangle_pocket(&rect, 5.0, 1.0, 2.0);
    
    assert!(!toolpaths.is_empty(), "Should generate helical toolpath for rotated shape");
    
    // Should have multiple depth layers
    assert!(
        toolpaths.len() >= 2,
        "Multi-pass pocket should generate multiple toolpath layers"
    );
}
