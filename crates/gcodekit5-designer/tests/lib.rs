#![allow(dead_code)]

mod designer_adaptive_integration;
mod designer_arrays_integration;
mod designer_dxf_integration;
mod designer_handle_drag_bug;
mod designer_import_integration;
mod designer_import_mirroring;
mod designer_integration;
// mod designer_mouse_event_mapping; // Removed
mod designer_pan_on_drag;
mod designer_parametric_integration;
mod designer_phase3_cam_ops;
mod designer_shift_key_snapping;
mod designer_state_integration;
mod designer_templates_test;
mod designer_vcarve_integration;
// mod designer_viewport_coordinate_mapping; // Removed

// New tests for rotation system fixes
mod rotation_system_tests;
// mod toolpath_rotation_tests; // Disabled due to API changes

// Phase 1 core coverage: designer operations integration tests
mod designer_operations_integration;
