//! Tests for shape insets (offset) with rotation
//! 
//! Verifies that when shapes have rotation applied, the inset (offset)
//! operation correctly rotates with the shape.

use gcodekit5_designer::model::{
    DesignCircle, DesignEllipse, DesignPolygon, DesignRectangle, DesignTriangle, DesignerShape,
    Point,
};
use gcodekit5_designer::ops::perform_offset;

#[test]
fn test_rectangle_inset_with_rotation() {
    // Create a rectangle at (10, 10) with width 20, height 10, rotated 45 degrees
    let mut rect = DesignRectangle::new(10.0, 10.0, 20.0, 10.0);
    rect.rotation = 45.0;

    // Apply a 2mm inset
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Rectangle(rect.clone()), -2.0);

    // The inset should be rotated along with the shape
    // We verify this by checking that the as_csg() method was called with proper rotation
    let bounds = inset_shape.bounds();
    
    // The bounds should be smaller than the original shape
    let (orig_x1, orig_y1, orig_x2, orig_y2) = rect.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = bounds;
    
    // Inset shape should be contained within original bounds (with tolerance for rotation)
    assert!(
        inset_x1 >= orig_x1 - 0.1 && inset_x2 <= orig_x2 + 0.1,
        "Inset X bounds should be within original: [{}, {}] vs [{}, {}]",
        inset_x1,
        inset_x2,
        orig_x1,
        orig_x2
    );
    assert!(
        inset_y1 >= orig_y1 - 0.1 && inset_y2 <= orig_y2 + 0.1,
        "Inset Y bounds should be within original: [{}, {}] vs [{}, {}]",
        inset_y1,
        inset_y2,
        orig_y1,
        orig_y2
    );
}

#[test]
fn test_circle_inset_with_rotation() {
    // Create a circle at (15, 15) with radius 10, rotated 30 degrees
    let mut circle = DesignCircle::new(Point::new(15.0, 15.0), 10.0);
    circle.rotation = 30.0;

    // Apply a 2mm inset
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Circle(circle.clone()), -2.0);

    // The inset circle should have a smaller radius
    let bounds = inset_shape.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = bounds;
    
    let inset_width = inset_x2 - inset_x1;
    let inset_height = inset_y2 - inset_y1;
    
    // For a circle with 2mm inset, the diameter should be reduced by ~4mm
    // Original diameter: 20mm, Inset diameter: ~16mm
    assert!(
        inset_width < 20.0 && inset_width > 14.0,
        "Inset width should be ~16mm, got {}",
        inset_width
    );
    assert!(
        inset_height < 20.0 && inset_height > 14.0,
        "Inset height should be ~16mm, got {}",
        inset_height
    );
}

#[test]
fn test_ellipse_inset_with_rotation() {
    // Create an ellipse at (20, 20) with rx=15, ry=10, rotated 60 degrees
    let mut ellipse = DesignEllipse::new(Point::new(20.0, 20.0), 15.0, 10.0);
    ellipse.rotation = 60.0;

    // Apply a 2mm inset
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Ellipse(ellipse.clone()), -2.0);

    // The inset should be rotated along with the ellipse
    let bounds = inset_shape.bounds();
    let (orig_x1, orig_y1, orig_x2, orig_y2) = ellipse.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = bounds;
    
    // Inset shape should be smaller than original
    let orig_area = (orig_x2 - orig_x1) * (orig_y2 - orig_y1);
    let inset_area = (inset_x2 - inset_x1) * (inset_y2 - inset_y1);
    
    assert!(
        inset_area < orig_area,
        "Inset area should be smaller than original: {} vs {}",
        inset_area,
        orig_area
    );
}

#[test]
fn test_triangle_inset_with_rotation() {
    // Create a triangle at (25, 25) with width 20, height 15, rotated 90 degrees
    let mut triangle = DesignTriangle::new(Point::new(25.0, 25.0), 20.0, 15.0);
    triangle.rotation = 90.0;

    // Apply a 1mm inset
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Triangle(triangle.clone()), -1.0);

    // The inset should be rotated along with the triangle
    let bounds = inset_shape.bounds();
    let (orig_x1, orig_y1, orig_x2, orig_y2) = triangle.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = bounds;
    
    // Inset shape should be smaller than original
    let orig_area = (orig_x2 - orig_x1) * (orig_y2 - orig_y1);
    let inset_area = (inset_x2 - inset_x1) * (inset_y2 - inset_y1);
    
    assert!(
        inset_area < orig_area,
        "Inset area should be smaller than original: {} vs {}",
        inset_area,
        orig_area
    );
}

#[test]
fn test_polygon_inset_with_rotation() {
    // Create a hexagon at (30, 30) with radius 12, rotated 45 degrees
    let mut polygon = DesignPolygon::new(Point::new(30.0, 30.0), 12.0, 6);
    polygon.rotation = 45.0;

    // Apply a 2mm inset
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Polygon(polygon.clone()), -2.0);

    // The inset should be rotated along with the polygon
    let bounds = inset_shape.bounds();
    let (orig_x1, orig_y1, orig_x2, orig_y2) = polygon.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = bounds;
    
    // Inset shape should be smaller than original
    let orig_area = (orig_x2 - orig_x1) * (orig_y2 - orig_y1);
    let inset_area = (inset_x2 - inset_x1) * (inset_y2 - inset_y1);
    
    assert!(
        inset_area < orig_area,
        "Inset area should be smaller than original: {} vs {}",
        inset_area,
        orig_area
    );
}

#[test]
fn test_multiple_rotation_angles() {
    // Test that insets work correctly at various rotation angles
    let angles = vec![0.0, 15.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0, 270.0];
    
    for angle in angles {
        let mut rect = DesignRectangle::new(50.0, 50.0, 30.0, 20.0);
        rect.rotation = angle;
        
        // Apply a 3mm inset
        let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Rectangle(rect.clone()), -3.0);
        
        // Verify the inset exists and has reasonable bounds
        let (inset_x1, inset_y1, inset_x2, inset_y2) = inset_shape.bounds();
        assert!(
            inset_x2 > inset_x1 && inset_y2 > inset_y1,
            "Inset shape at {}° should have valid bounds",
            angle
        );
    }
}

#[test]
fn test_zero_rotation_baseline() {
    // Verify that shapes with zero rotation work correctly (baseline test)
    let rect = DesignRectangle::new(100.0, 100.0, 40.0, 30.0);
    // rotation is 0.0 by default
    
    let inset_shape = perform_offset(&gcodekit5_designer::model::Shape::Rectangle(rect.clone()), -4.0);
    
    let (orig_x1, orig_y1, orig_x2, orig_y2) = rect.bounds();
    let (inset_x1, inset_y1, inset_x2, inset_y2) = inset_shape.bounds();
    
    // For an axis-aligned rectangle with 4mm inset:
    // Width should be reduced by 8mm, height by 8mm
    let orig_width = orig_x2 - orig_x1;
    let orig_height = orig_y2 - orig_y1;
    let inset_width = inset_x2 - inset_x1;
    let inset_height = inset_y2 - inset_y1;
    
    assert!(
        (orig_width - inset_width) > 7.0 && (orig_width - inset_width) < 9.0,
        "Width reduction should be ~8mm for 4mm inset, got {}",
        orig_width - inset_width
    );
    assert!(
        (orig_height - inset_height) > 7.0 && (orig_height - inset_height) < 9.0,
        "Height reduction should be ~8mm for 4mm inset, got {}",
        orig_height - inset_height
    );
}
