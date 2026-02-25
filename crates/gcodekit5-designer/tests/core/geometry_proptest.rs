// Property-based tests for geometry operations using proptest

use gcodekit5_designer::model::{rotate_point, DesignerShape};
use gcodekit5_designer::spatial_index::{Bounds, SpatialIndex};
use gcodekit5_designer::{Circle, Ellipse, Line, Point, Rectangle};
use proptest::prelude::*;

// ── Custom strategies ─────────────────────────────────────────────────

fn arb_point() -> impl Strategy<Value = Point> {
    (-1000.0..1000.0_f64, -1000.0..1000.0_f64).prop_map(|(x, y)| Point::new(x, y))
}

fn arb_bounds() -> impl Strategy<Value = Bounds> {
    (
        -500.0..500.0_f64,
        -500.0..500.0_f64,
        1.0..200.0_f64,
        1.0..200.0_f64,
    )
        .prop_map(|(x, y, w, h)| Bounds::new(x, y, x + w, y + h))
}

fn arb_positive_f64() -> impl Strategy<Value = f64> {
    1.0..500.0_f64
}

fn arb_angle_deg() -> impl Strategy<Value = f64> {
    -360.0..360.0_f64
}

// ── Point distance properties ─────────────────────────────────────────

proptest! {
    #[test]
    fn point_distance_is_symmetric(a in arb_point(), b in arb_point()) {
        let d_ab = a.distance_to(&b);
        let d_ba = b.distance_to(&a);
        prop_assert!((d_ab - d_ba).abs() < 1e-10,
            "distance should be symmetric: d(a,b)={} != d(b,a)={}", d_ab, d_ba);
    }

    #[test]
    fn point_distance_self_is_zero(p in arb_point()) {
        let d = p.distance_to(&p);
        prop_assert!(d.abs() < 1e-10, "distance to self should be 0, got {}", d);
    }

    #[test]
    fn point_distance_is_non_negative(a in arb_point(), b in arb_point()) {
        let d = a.distance_to(&b);
        prop_assert!(d >= 0.0, "distance should be non-negative, got {}", d);
    }

    #[test]
    fn point_distance_triangle_inequality(
        a in arb_point(),
        b in arb_point(),
        c in arb_point()
    ) {
        let d_ab = a.distance_to(&b);
        let d_bc = b.distance_to(&c);
        let d_ac = a.distance_to(&c);
        prop_assert!(d_ac <= d_ab + d_bc + 1e-9,
            "triangle inequality violated: d(a,c)={} > d(a,b)+d(b,c)={}",
            d_ac, d_ab + d_bc);
    }
}

// ── rotate_point properties ───────────────────────────────────────────

proptest! {
    #[test]
    fn rotate_preserves_distance_to_center(
        p in arb_point(),
        center in arb_point(),
        angle in arb_angle_deg()
    ) {
        let rotated = rotate_point(p, center, angle);
        let d_before = p.distance_to(&center);
        let d_after = rotated.distance_to(&center);
        prop_assert!((d_before - d_after).abs() < 1e-6,
            "rotation should preserve distance to center: before={}, after={}",
            d_before, d_after);
    }

    #[test]
    fn rotate_360_returns_to_original(
        p in arb_point(),
        center in arb_point()
    ) {
        let rotated = rotate_point(p, center, 360.0);
        prop_assert!((rotated.x - p.x).abs() < 1e-6, "x mismatch after 360°");
        prop_assert!((rotated.y - p.y).abs() < 1e-6, "y mismatch after 360°");
    }

    #[test]
    fn rotate_inverse_returns_to_original(
        p in arb_point(),
        center in arb_point(),
        angle in arb_angle_deg()
    ) {
        let rotated = rotate_point(p, center, angle);
        let back = rotate_point(rotated, center, -angle);
        prop_assert!((back.x - p.x).abs() < 1e-4,
            "x mismatch after rotate+inverse: {} vs {}", back.x, p.x);
        prop_assert!((back.y - p.y).abs() < 1e-4,
            "y mismatch after rotate+inverse: {} vs {}", back.y, p.y);
    }
}

// ── Bounds properties ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn bounds_intersects_is_symmetric(a in arb_bounds(), b in arb_bounds()) {
        prop_assert_eq!(a.intersects(&b), b.intersects(&a),
            "intersects should be symmetric");
    }

    #[test]
    fn bounds_self_intersects(b in arb_bounds()) {
        prop_assert!(b.intersects(&b), "bounds should intersect itself");
    }

    #[test]
    fn bounds_contains_own_center(b in arb_bounds()) {
        let (cx, cy) = b.center();
        prop_assert!(b.contains_point(cx, cy),
            "bounds should contain its own center ({}, {})", cx, cy);
    }

    #[test]
    fn bounds_width_height_positive(b in arb_bounds()) {
        prop_assert!(b.width() > 0.0, "width should be positive");
        prop_assert!(b.height() > 0.0, "height should be positive");
    }

    #[test]
    fn bounds_contains_corners(b in arb_bounds()) {
        prop_assert!(b.contains_point(b.min_x, b.min_y));
        prop_assert!(b.contains_point(b.max_x, b.max_y));
        prop_assert!(b.contains_point(b.min_x, b.max_y));
        prop_assert!(b.contains_point(b.max_x, b.min_y));
    }

    #[test]
    fn bounds_does_not_contain_outside_point(
        b in arb_bounds(),
        offset in 1.0..100.0_f64
    ) {
        // A point beyond max_x + offset should not be contained
        prop_assert!(!b.contains_point(b.max_x + offset, b.min_y));
    }

    #[test]
    fn bounds_contains_bounds_implies_intersects(
        a in arb_bounds(),
        inner_offset_x in 0.1..0.4_f64,
        inner_offset_y in 0.1..0.4_f64,
    ) {
        // Create a smaller bounds inside `a`
        let w = a.width() * inner_offset_x;
        let h = a.height() * inner_offset_y;
        let (cx, cy) = a.center();
        let inner = Bounds::new(cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0);
        prop_assert!(a.contains_bounds(&inner),
            "should contain inner bounds");
        prop_assert!(a.intersects(&inner),
            "containment implies intersection");
    }
}

// ── SpatialIndex properties ───────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn spatial_index_insert_then_find(
        item_bounds in arb_bounds()
    ) {
        let mut index = SpatialIndex::default();
        index.insert(42, &item_bounds);
        let (cx, cy) = item_bounds.center();
        let results = index.query_point(cx, cy);
        prop_assert!(results.contains(&42),
            "inserted item should be found at its center");
    }

    #[test]
    fn spatial_index_query_subset(
        items in proptest::collection::vec(arb_bounds(), 2..10)
    ) {
        let mut index = SpatialIndex::default();
        for (i, b) in items.iter().enumerate() {
            index.insert(i as u64, b);
        }

        // Query the full world should return all items
        let big_query = Bounds::new(-100000.0, -100000.0, 100000.0, 100000.0);
        let all_results = index.query(&big_query);
        prop_assert_eq!(all_results.len(), items.len(),
            "querying full world should find all items");
    }

    #[test]
    fn spatial_index_remove_then_not_found(
        item_bounds in arb_bounds()
    ) {
        let mut index = SpatialIndex::default();
        index.insert(99, &item_bounds);
        index.remove(99, &item_bounds);
        let (cx, cy) = item_bounds.center();
        let results = index.query_point(cx, cy);
        prop_assert!(!results.contains(&99),
            "removed item should not be found");
    }
}

// ── Shape bounds invariants ───────────────────────────────────────────

proptest! {
    #[test]
    fn rectangle_bounds_contain_shape(
        cx in -200.0..200.0_f64,
        cy in -200.0..200.0_f64,
        w in arb_positive_f64(),
        h in arb_positive_f64()
    ) {
        let rect = Rectangle::new(cx, cy, w, h);
        let (min_x, min_y, max_x, max_y) = rect.bounds();
        prop_assert!(min_x <= max_x);
        prop_assert!(min_y <= max_y);
        // bounds() uses lyon f32 internally, so allow for f32 precision loss
        prop_assert!((max_x - min_x) >= w - 0.01,
            "bounds width {} < shape width {}", max_x - min_x, w);
        prop_assert!((max_y - min_y) >= h - 0.01,
            "bounds height {} < shape height {}", max_y - min_y, h);
    }

    #[test]
    fn circle_bounds_contain_center(
        cx in -200.0..200.0_f64,
        cy in -200.0..200.0_f64,
        r in 1.0..100.0_f64
    ) {
        let circle = Circle::new(Point::new(cx, cy), r);
        let (min_x, min_y, max_x, max_y) = circle.bounds();
        prop_assert!(min_x <= cx && cx <= max_x,
            "center x {} not in bounds [{}, {}]", cx, min_x, max_x);
        prop_assert!(min_y <= cy && cy <= max_y,
            "center y {} not in bounds [{}, {}]", cy, min_y, max_y);
    }

    #[test]
    fn circle_bounds_size_matches_diameter(
        cx in -200.0..200.0_f64,
        cy in -200.0..200.0_f64,
        r in 1.0..100.0_f64
    ) {
        let circle = Circle::new(Point::new(cx, cy), r);
        let (min_x, min_y, max_x, max_y) = circle.bounds();
        let bw = max_x - min_x;
        let bh = max_y - min_y;
        prop_assert!((bw - 2.0 * r).abs() < 1e-6,
            "bounds width {} != diameter {}", bw, 2.0 * r);
        prop_assert!((bh - 2.0 * r).abs() < 1e-6,
            "bounds height {} != diameter {}", bh, 2.0 * r);
    }

    #[test]
    fn line_bounds_contain_endpoints(
        x1 in -200.0..200.0_f64,
        y1 in -200.0..200.0_f64,
        x2 in -200.0..200.0_f64,
        y2 in -200.0..200.0_f64
    ) {
        let line = Line::new(Point::new(x1, y1), Point::new(x2, y2));
        let (min_x, min_y, max_x, max_y) = line.bounds();
        // bounds() uses lyon f32 path rendering with rotation applied,
        // so allow 0.1 tolerance for f32 precision loss
        let tol = 0.1;
        prop_assert!(min_x <= x1 + tol && x1 <= max_x + tol);
        prop_assert!(min_x <= x2 + tol && x2 <= max_x + tol);
        prop_assert!(min_y <= y1 + tol && y1 <= max_y + tol);
        prop_assert!(min_y <= y2 + tol && y2 <= max_y + tol);
    }

    #[test]
    fn ellipse_bounds_contain_center(
        cx in -200.0..200.0_f64,
        cy in -200.0..200.0_f64,
        rx in 1.0..100.0_f64,
        ry in 1.0..100.0_f64
    ) {
        let ellipse = Ellipse::new(Point::new(cx, cy), rx, ry);
        let (min_x, min_y, max_x, max_y) = ellipse.bounds();
        prop_assert!(min_x <= cx && cx <= max_x);
        prop_assert!(min_y <= cy && cy <= max_y);
    }
}

// ── Shape translate invariant ─────────────────────────────────────────

proptest! {
    #[test]
    fn translate_shifts_bounds(
        cx in -100.0..100.0_f64,
        cy in -100.0..100.0_f64,
        w in 10.0..100.0_f64,
        h in 10.0..100.0_f64,
        dx in -50.0..50.0_f64,
        dy in -50.0..50.0_f64
    ) {
        let rect = Rectangle::new(cx, cy, w, h);
        let (ox1, oy1, ox2, oy2) = rect.bounds();
        let mut rect2 = rect.clone();
        rect2.translate(dx, dy);
        let (nx1, ny1, nx2, ny2) = rect2.bounds();
        // translate goes through f32 Transform, so allow f32 precision tolerance
        let tol = 0.01;
        prop_assert!((nx1 - (ox1 + dx)).abs() < tol,
            "min_x shift: {} vs expected {}", nx1, ox1 + dx);
        prop_assert!((ny1 - (oy1 + dy)).abs() < tol,
            "min_y shift: {} vs expected {}", ny1, oy1 + dy);
        prop_assert!((nx2 - (ox2 + dx)).abs() < tol,
            "max_x shift: {} vs expected {}", nx2, ox2 + dx);
        prop_assert!((ny2 - (oy2 + dy)).abs() < tol,
            "max_y shift: {} vs expected {}", ny2, oy2 + dy);
    }
}
