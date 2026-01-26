//! Properties panel for the designer.
//!
//! This module provides the property editing panel shown on the right side of the designer.
//! It's organized into:
//! - Main panel orchestration (this file)
//! - Handlers for different property categories (handlers/)

mod handlers;

use crate::t;
use gcodekit5_core::units;
use gcodekit5_designer::designer_state::DesignerState;
use gcodekit5_designer::font_manager;
use gcodekit5_designer::model::{DesignerShape, Shape};
use gcodekit5_designer::pocket_operations::PocketStrategy;
use gcodekit5_designer::shapes::OperationType;
use gcodekit5_settings::SettingsPersistence;
use gtk4::prelude::*;
use gtk4::{
    Box, CheckButton, DropDown, Entry, EventControllerFocus, Expression, Frame, Label, Orientation,
    ScrolledWindow, StringList,
};
use std::cell::RefCell;
use std::rc::Rc;

const MM_PER_PT: f64 = 25.4 / 72.0;

fn mm_to_pt(mm: f64) -> f64 {
    mm / MM_PER_PT
}

fn format_font_points(mm: f64) -> String {
    format!("{:.2}", mm_to_pt(mm))
}

/// Properties panel showing editable properties for selected shapes.
pub struct PropertiesPanel {
    pub widget: ScrolledWindow,
    state: Rc<RefCell<DesignerState>>,
    settings: Rc<RefCell<SettingsPersistence>>,
    preview_shapes: Rc<RefCell<Vec<Shape>>>,
    _content: Box,
    header: Label,

    // Sections (visibility controlled)
    pos_frame: Frame,
    size_frame: Frame,
    rot_frame: Frame,
    corner_frame: Frame,
    text_frame: Frame,
    cam_frame: Frame,
    ops_frame: Frame,
    empty_label: Label,

    // Property widgets
    pos_x_entry: Entry,
    pos_y_entry: Entry,
    width_entry: Entry,
    height_entry: Entry,
    lock_aspect_ratio: CheckButton,
    rotation_entry: Entry,
    // Rectangle widgets
    corner_radius_entry: Entry,
    is_slot_check: CheckButton,
    // Text widgets
    text_entry: Entry,
    font_family_combo: DropDown,
    font_bold_check: CheckButton,
    font_italic_check: CheckButton,
    font_size_entry: Entry,
    // Polygon widgets
    polygon_frame: Frame,
    sides_entry: Entry,

    // Gear widgets
    gear_frame: Frame,
    gear_module_entry: Entry,
    gear_teeth_entry: Entry,
    gear_pressure_angle_entry: Entry,

    // Sprocket widgets
    sprocket_frame: Frame,
    sprocket_pitch_entry: Entry,
    sprocket_teeth_entry: Entry,
    sprocket_roller_diameter_entry: Entry,

    // CAM widgets
    op_type_combo: DropDown,
    depth_entry: Entry,
    step_down_entry: Entry,
    step_in_entry: Entry,
    ramp_angle_entry: Entry,
    strategy_combo: DropDown,
    raster_fill_entry: Entry,

    // Geometry Ops widgets
    offset_entry: Entry,
    fillet_entry: Entry,
    chamfer_entry: Entry,

    // Unit Labels
    x_unit_label: Label,
    y_unit_label: Label,
    width_unit_label: Label,
    height_unit_label: Label,
    radius_unit_label: Label,
    font_size_unit_label: Label,
    depth_unit_label: Label,
    step_down_unit_label: Label,
    step_in_unit_label: Label,
    offset_unit_label: Label,
    fillet_unit_label: Label,
    chamfer_unit_label: Label,
    // Redraw callback
    redraw_callback: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    // Flag to prevent feedback loops during updates
    updating: Rc<RefCell<bool>>,
    // Flag to track if any widget has focus (being edited)
    has_focus: Rc<RefCell<bool>>,
    // Aspect ratio (width/height) for locked resizing
    aspect_ratio: Rc<RefCell<f64>>,
}

impl PropertiesPanel {
    /// Create a new properties panel.
    pub fn new(
        state: Rc<RefCell<DesignerState>>,
        settings: Rc<RefCell<SettingsPersistence>>,
        preview_shapes: Rc<RefCell<Vec<Shape>>>,
    ) -> Rc<Self> {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .width_request(280)
            .hexpand(false)
            .build();

        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);

        // Header (kept for internal state, not shown in UI)
        let header = Label::new(Some(&t!("Properties")));
        header.add_css_class("title-3");
        header.add_css_class("heading");
        header.set_halign(gtk4::Align::Start);
        header.set_visible(false);

        // Build all UI sections
        let (pos_frame, pos_x_entry, pos_y_entry, x_unit_label, y_unit_label) =
            Self::build_position_section();
        content.append(&pos_frame);

        let (
            size_frame,
            width_entry,
            height_entry,
            lock_aspect_ratio,
            width_unit_label,
            height_unit_label,
        ) = Self::build_size_section();
        content.append(&size_frame);

        let (rot_frame, rotation_entry) = Self::build_rotation_section();
        content.append(&rot_frame);

        let (corner_frame, corner_radius_entry, is_slot_check, radius_unit_label) =
            Self::build_corner_section();
        content.append(&corner_frame);

        let (
            text_frame,
            text_entry,
            font_family_combo,
            font_bold_check,
            font_italic_check,
            font_size_entry,
            font_size_unit_label,
        ) = Self::build_text_section();
        content.append(&text_frame);

        let (polygon_frame, sides_entry) = Self::build_polygon_section();
        content.append(&polygon_frame);

        let (gear_frame, gear_module_entry, gear_teeth_entry, gear_pressure_angle_entry) =
            Self::build_gear_section();
        content.append(&gear_frame);

        let (
            sprocket_frame,
            sprocket_pitch_entry,
            sprocket_teeth_entry,
            sprocket_roller_diameter_entry,
        ) = Self::build_sprocket_section();
        content.append(&sprocket_frame);

        let (
            ops_frame,
            offset_entry,
            fillet_entry,
            chamfer_entry,
            offset_unit_label,
            fillet_unit_label,
            chamfer_unit_label,
        ) = Self::build_geometry_ops_section();
        content.append(&ops_frame);

        let (
            cam_frame,
            op_type_combo,
            depth_entry,
            step_down_entry,
            step_in_entry,
            ramp_angle_entry,
            strategy_combo,
            raster_fill_entry,
            depth_unit_label,
            step_down_unit_label,
            step_in_unit_label,
        ) = Self::build_cam_section();
        content.append(&cam_frame);

        // Empty state message
        let empty_label = Label::new(Some(&t!("Select a shape to edit its properties")));
        empty_label.add_css_class("dim-label");
        empty_label.set_wrap(true);
        empty_label.set_margin_top(24);
        content.append(&empty_label);

        scrolled.set_child(Some(&content));

        let panel = Rc::new(Self {
            widget: scrolled,
            state: state.clone(),
            settings: settings.clone(),
            preview_shapes: preview_shapes.clone(),
            _content: content,
            pos_frame,
            size_frame,
            rot_frame,
            corner_frame,
            text_frame,
            polygon_frame,
            gear_frame,
            sprocket_frame,
            cam_frame,
            ops_frame,
            empty_label,
            pos_x_entry,
            pos_y_entry,
            width_entry,
            height_entry,
            rotation_entry,
            corner_radius_entry,
            is_slot_check,
            text_entry,
            font_family_combo,
            font_bold_check,
            font_italic_check,
            font_size_entry,
            sides_entry,
            gear_module_entry,
            gear_teeth_entry,
            gear_pressure_angle_entry,
            sprocket_pitch_entry,
            sprocket_teeth_entry,
            sprocket_roller_diameter_entry,
            op_type_combo,
            depth_entry,
            step_down_entry,
            step_in_entry,
            ramp_angle_entry,
            strategy_combo,
            raster_fill_entry,
            offset_entry,
            fillet_entry,
            chamfer_entry,
            header,
            x_unit_label,
            y_unit_label,
            width_unit_label,
            height_unit_label,
            radius_unit_label,
            font_size_unit_label,
            depth_unit_label,
            step_down_unit_label,
            step_in_unit_label,
            offset_unit_label,
            fillet_unit_label,
            chamfer_unit_label,
            lock_aspect_ratio,
            redraw_callback: Rc::new(RefCell::new(None)),
            updating: Rc::new(RefCell::new(false)),
            has_focus: Rc::new(RefCell::new(bool::default())),
            aspect_ratio: Rc::new(RefCell::new(1.0)),
        });

        // Connect value change handlers
        panel.setup_handlers();

        // Setup focus tracking for all spin buttons
        panel.setup_focus_tracking();

        panel
    }

    /// Set the callback to redraw the canvas.
    pub fn set_redraw_callback<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.redraw_callback.borrow_mut() = Some(Rc::new(callback));
    }

    fn create_section(title: &str) -> Frame {
        Frame::new(Some(title))
    }

    /// Setup all property change handlers using modular handler functions.
    fn setup_handlers(&self) {
        // Dimension handlers
        handlers::dimensions::setup_position_x_handler(
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.width_entry,
            &self.height_entry,
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::dimensions::setup_position_y_handler(
            &self.pos_y_entry,
            &self.pos_x_entry,
            &self.width_entry,
            &self.height_entry,
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::dimensions::setup_width_activate_handler(
            &self.width_entry,
            &self.height_entry,
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.lock_aspect_ratio,
            self.aspect_ratio.clone(),
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::dimensions::setup_height_activate_handler(
            &self.height_entry,
            &self.width_entry,
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.lock_aspect_ratio,
            self.aspect_ratio.clone(),
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::dimensions::setup_lock_aspect_handler(
            &self.lock_aspect_ratio,
            &self.width_entry,
            &self.height_entry,
            self.aspect_ratio.clone(),
            self.state.clone(),
            self.settings.clone(),
        );

        handlers::dimensions::setup_width_focus_out_handler(
            &self.width_entry,
            &self.height_entry,
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.lock_aspect_ratio,
            self.aspect_ratio.clone(),
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::dimensions::setup_height_focus_out_handler(
            &self.height_entry,
            &self.width_entry,
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.lock_aspect_ratio,
            self.aspect_ratio.clone(),
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        // Geometry handlers
        handlers::geometry::setup_rotation_handler(
            &self.rotation_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::geometry::setup_corner_radius_handler(
            &self.corner_radius_entry,
            self.state.clone(),
            self.settings.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::geometry::setup_is_slot_handler(
            &self.is_slot_check,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::geometry::setup_sides_handler(
            &self.sides_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        // Text handlers
        handlers::text::setup_text_content_handler(
            &self.text_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::text::setup_font_size_handler(
            &self.font_size_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::text::setup_font_family_handler(
            &self.font_family_combo,
            &self.font_bold_check,
            &self.font_italic_check,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::text::setup_font_bold_handler(
            &self.font_bold_check,
            &self.font_family_combo,
            &self.font_italic_check,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::text::setup_font_italic_handler(
            &self.font_italic_check,
            &self.font_family_combo,
            &self.font_bold_check,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        // CAM handlers
        handlers::cam::setup_operation_type_handler(
            &self.op_type_combo,
            self.state.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_depth_handler(
            &self.depth_entry,
            &self.op_type_combo,
            self.state.clone(),
            self.settings.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_step_down_handler(
            &self.step_down_entry,
            self.state.clone(),
            self.settings.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_step_in_handler(
            &self.step_in_entry,
            self.state.clone(),
            self.settings.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_raster_fill_handler(
            &self.raster_fill_entry,
            self.state.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_ramp_angle_handler(
            &self.ramp_angle_entry,
            self.state.clone(),
            self.updating.clone(),
        );

        handlers::cam::setup_strategy_handler(
            &self.strategy_combo,
            self.state.clone(),
            self.updating.clone(),
        );

        // Gear/Sprocket handlers
        handlers::gear_sprocket::setup_gear_module_handler(
            &self.gear_module_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::gear_sprocket::setup_gear_teeth_handler(
            &self.gear_teeth_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::gear_sprocket::setup_gear_pressure_angle_handler(
            &self.gear_pressure_angle_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::gear_sprocket::setup_sprocket_pitch_handler(
            &self.sprocket_pitch_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::gear_sprocket::setup_sprocket_teeth_handler(
            &self.sprocket_teeth_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        handlers::gear_sprocket::setup_sprocket_roller_diameter_handler(
            &self.sprocket_roller_diameter_entry,
            self.state.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
        );

        // Effects handlers
        handlers::effects::setup_offset_handler(
            &self.offset_entry,
            self.state.clone(),
            self.preview_shapes.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
            self.has_focus.clone(),
        );

        handlers::effects::setup_fillet_handler(
            &self.fillet_entry,
            self.state.clone(),
            self.preview_shapes.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
            self.has_focus.clone(),
        );

        handlers::effects::setup_chamfer_handler(
            &self.chamfer_entry,
            self.state.clone(),
            self.preview_shapes.clone(),
            self.redraw_callback.clone(),
            self.updating.clone(),
            self.has_focus.clone(),
        );
    }

    fn set_entry_text_if_changed(
        &self,
        entry: &Entry,
        new_value: f32,
        system: gcodekit5_core::units::MeasurementSystem,
    ) {
        if let Ok(current_parsed) = units::parse_length(&entry.text(), system) {
            if (current_parsed as f64 - new_value as f64).abs() > 1e-6 {
                entry.set_text(&units::format_length(new_value, system));
            }
        } else {
            entry.set_text(&units::format_length(new_value, system));
        }
    }

    /// Update the panel from the current selection.
    pub fn update_from_selection(&self) {
        // Don't update if any widget has focus (user is editing)
        if *self.has_focus.borrow() {
            return;
        }

        // Get current measurement system
        let system = self.settings.borrow().config().ui.measurement_system;
        let unit_label = units::get_unit_label(system);

        // Update unit labels
        self.x_unit_label.set_text(unit_label);
        self.y_unit_label.set_text(unit_label);
        self.width_unit_label.set_text(unit_label);
        self.height_unit_label.set_text(unit_label);
        self.radius_unit_label.set_text(unit_label);
        self.font_size_unit_label.set_text("pt");
        self.depth_unit_label.set_text(unit_label);
        self.step_down_unit_label.set_text(unit_label);
        self.step_in_unit_label.set_text(unit_label);
        self.offset_unit_label.set_text(unit_label);
        self.fillet_unit_label.set_text(unit_label);
        self.chamfer_unit_label.set_text(unit_label);

        // Extract data first to avoid holding the borrow while updating widgets
        let selection_data = {
            let designer_state = self.state.borrow();
            let selected: Vec<_> = designer_state
                .canvas
                .shapes()
                .filter(|s| s.selected)
                .collect();

            if selected.is_empty() {
                None
            } else if selected.len() == 1 {
                // Single selection - show all properties
                let obj = &selected[0];
                let any_not_text = !matches!(obj.shape, Shape::Text(_));
                Some((
                    vec![obj.id],
                    Some(obj.shape.clone()),
                    obj.operation_type,
                    obj.pocket_depth,
                    obj.step_down,
                    obj.step_in,
                    obj.ramp_angle,
                    obj.pocket_strategy,
                    obj.raster_fill_ratio,
                    obj.offset,
                    obj.fillet,
                    obj.chamfer,
                    any_not_text,
                    obj.lock_aspect_ratio,
                ))
            } else {
                // Multiple selection - only show CAM properties (use first shape's values)
                let obj = &selected[0];
                let any_not_text = selected.iter().any(|s| !matches!(s.shape, Shape::Text(_)));
                Some((
                    selected.iter().map(|s| s.id).collect(),
                    None, // No shape data for multi-selection
                    obj.operation_type,
                    obj.pocket_depth,
                    obj.step_down,
                    obj.step_in,
                    obj.ramp_angle,
                    obj.pocket_strategy,
                    obj.raster_fill_ratio,
                    obj.offset,
                    obj.fillet,
                    obj.chamfer,
                    any_not_text,
                    false, // Multi-selection: don't lock aspect ratio
                ))
            }
        };

        if let Some((
            ids,
            shape_opt,
            op_type,
            depth,
            step_down,
            step_in,
            ramp_angle,
            strategy,
            raster_fill,
            offset,
            fillet,
            chamfer,
            any_not_text,
            lock_aspect,
        )) = selection_data
        {
            // Set flag to prevent feedback loop during updates
            *self.updating.borrow_mut() = true;

            // Update header with shape ID(s)
            if ids.len() == 1 {
                self.header
                    .set_text(&format!("{} [{}]", t!("Properties"), ids[0]));
            } else {
                self.header.set_text(&format!(
                    "{} [{} {}]",
                    t!("Properties"),
                    ids.len(),
                    t!("shapes")
                ));
            }

            // Show/hide appropriate sections
            self.empty_label.set_visible(false);
            self.cam_frame.set_visible(true);
            self.ops_frame.set_visible(any_not_text);

            if let Some(shape) = shape_opt {
                // Single selection - show shape-specific properties
                self.pos_frame.set_visible(true);
                self.size_frame.set_visible(true);
                self.rot_frame.set_visible(true);

                // Update position and size using bounding box
                let (min_x, min_y, max_x, max_y) = shape.bounds();
                self.set_entry_text_if_changed(&self.pos_x_entry, min_x as f32, system);
                self.set_entry_text_if_changed(&self.pos_y_entry, min_y as f32, system);
                self.set_entry_text_if_changed(&self.width_entry, (max_x - min_x) as f32, system);
                self.set_entry_text_if_changed(&self.height_entry, (max_y - min_y) as f32, system);
                self.lock_aspect_ratio.set_active(lock_aspect);

                // Shape-specific properties
                match &shape {
                    Shape::Rectangle(r) => {
                        self.corner_frame.set_visible(true);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.set_entry_text_if_changed(
                            &self.corner_radius_entry,
                            r.corner_radius as f32,
                            system,
                        );
                        self.is_slot_check.set_active(r.is_slot);
                        self.rotation_entry.set_text(&format!("{:.1}", r.rotation));
                    }
                    Shape::Circle(_) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.rotation_entry.set_text("0.0");
                    }
                    Shape::Ellipse(e) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.rotation_entry
                            .set_text(&format!("{:.1}", e.rotation.to_degrees()));
                    }
                    Shape::Text(t) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(true);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.text_entry.set_text(&t.text);
                        self.font_size_entry
                            .set_text(&format_font_points(t.font_size));
                        self.font_bold_check.set_active(t.bold);
                        self.font_italic_check.set_active(t.italic);

                        // Set font family in dropdown
                        let Some(model) =
                            self.font_family_combo.model().and_downcast::<StringList>()
                        else {
                            return;
                        };
                        for i in 0..model.n_items() {
                            if let Some(item) = model.string(i) {
                                if item == t.font_family {
                                    self.font_family_combo.set_selected(i);
                                    break;
                                }
                            }
                        }
                        self.rotation_entry.set_text("0.0");
                    }
                    Shape::Polygon(p) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(true);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.sides_entry.set_text(&p.sides.to_string());
                        self.rotation_entry
                            .set_text(&format!("{:.1}", p.rotation.to_degrees()));
                    }
                    Shape::Gear(g) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(true);
                        self.sprocket_frame.set_visible(false);
                        self.gear_module_entry.set_text(&format!("{:.2}", g.module));
                        self.gear_teeth_entry.set_text(&g.teeth.to_string());
                        self.gear_pressure_angle_entry
                            .set_text(&format!("{:.1}", g.pressure_angle.to_degrees()));
                        self.rotation_entry.set_text("0.0");
                    }
                    Shape::Sprocket(s) => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(true);
                        self.sprocket_pitch_entry
                            .set_text(&format!("{:.2}", s.pitch));
                        self.sprocket_teeth_entry.set_text(&s.teeth.to_string());
                        self.sprocket_roller_diameter_entry
                            .set_text(&format!("{:.2}", s.roller_diameter));
                        self.rotation_entry.set_text("0.0");
                    }
                    _ => {
                        self.corner_frame.set_visible(false);
                        self.text_frame.set_visible(false);
                        self.polygon_frame.set_visible(false);
                        self.gear_frame.set_visible(false);
                        self.sprocket_frame.set_visible(false);
                        self.rotation_entry.set_text("0.0");
                    }
                }
            } else {
                // Multi-selection - hide shape-specific properties, show common props
                self.pos_frame.set_visible(true);
                self.size_frame.set_visible(true);
                self.rot_frame.set_visible(false);
                self.corner_frame.set_visible(false);
                self.text_frame.set_visible(false);
                self.polygon_frame.set_visible(false);
                self.gear_frame.set_visible(false);
                self.sprocket_frame.set_visible(false);

                // Calculate bounding box of all selected shapes
                let designer_state = self.state.borrow();
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for shape in designer_state.canvas.shapes().filter(|s| s.selected) {
                    let (x1, y1, x2, y2) = shape.shape.bounds();
                    min_x = min_x.min(x1);
                    min_y = min_y.min(y1);
                    max_x = max_x.max(x2);
                    max_y = max_y.max(y2);
                }

                self.set_entry_text_if_changed(&self.pos_x_entry, min_x as f32, system);
                self.set_entry_text_if_changed(&self.pos_y_entry, min_y as f32, system);
                self.set_entry_text_if_changed(&self.width_entry, (max_x - min_x) as f32, system);
                self.set_entry_text_if_changed(&self.height_entry, (max_y - min_y) as f32, system);
            }

            // Update CAM properties (common to all shapes)
            self.op_type_combo
                .set_selected(if op_type == OperationType::Pocket {
                    1
                } else {
                    0
                });
            self.set_entry_text_if_changed(&self.depth_entry, depth as f32, system);
            self.set_entry_text_if_changed(&self.step_down_entry, step_down as f32, system);
            self.set_entry_text_if_changed(&self.step_in_entry, step_in as f32, system);
            self.ramp_angle_entry
                .set_text(&format!("{:.1}", ramp_angle));

            let strategy_index = match strategy {
                PocketStrategy::Raster { .. } => 0,
                PocketStrategy::ContourParallel => 1,
                PocketStrategy::Adaptive => 2,
            };
            self.strategy_combo.set_selected(strategy_index);
            self.raster_fill_entry
                .set_text(&format!("{:.0}", raster_fill * 100.0));

            // Update geometry ops values
            self.offset_entry.set_text(&format!("{:.2}", offset));
            self.fillet_entry.set_text(&format!("{:.2}", fillet));
            self.chamfer_entry.set_text(&format!("{:.2}", chamfer));

            // Enable/disable pocket-specific controls
            let is_pocket = op_type == OperationType::Pocket;
            self.strategy_combo.set_sensitive(is_pocket);
            self.step_in_entry.set_sensitive(is_pocket);
            self.raster_fill_entry.set_sensitive(is_pocket);

            *self.updating.borrow_mut() = false;
        } else {
            // Nothing selected - show empty state
            self.empty_label.set_visible(true);
            self.pos_frame.set_visible(false);
            self.size_frame.set_visible(false);
            self.rot_frame.set_visible(false);
            self.corner_frame.set_visible(false);
            self.text_frame.set_visible(false);
            self.polygon_frame.set_visible(false);
            self.gear_frame.set_visible(false);
            self.sprocket_frame.set_visible(false);
            self.cam_frame.set_visible(false);
            self.ops_frame.set_visible(false);
            self.header.set_text(&t!("Properties"));

            // Clear entries
            *self.updating.borrow_mut() = true;
            self.pos_x_entry.set_text("");
            self.pos_y_entry.set_text("");
            self.width_entry.set_text("");
            self.height_entry.set_text("");
            self.rotation_entry.set_text("");
            self.corner_radius_entry.set_text("");
            self.depth_entry.set_text("");
            self.step_down_entry.set_text("");
            self.step_in_entry.set_text("");
            self.ramp_angle_entry.set_text("");

            // Disable widgets when nothing selected
            self.op_type_combo.set_sensitive(false);
            self.depth_entry.set_sensitive(false);
            self.step_down_entry.set_sensitive(false);
            self.step_in_entry.set_sensitive(false);
            self.ramp_angle_entry.set_sensitive(false);
            self.strategy_combo.set_sensitive(false);
            self.raster_fill_entry.set_sensitive(false);

            self.raster_fill_entry.set_text("");
            *self.updating.borrow_mut() = false;
        }
    }

    fn setup_focus_tracking(&self) {
        // Track focus for all entries to prevent updates while user is editing
        let entries = vec![
            &self.pos_x_entry,
            &self.pos_y_entry,
            &self.width_entry,
            &self.height_entry,
            &self.rotation_entry,
            &self.corner_radius_entry,
            &self.font_size_entry,
            &self.depth_entry,
            &self.step_down_entry,
            &self.step_in_entry,
            &self.ramp_angle_entry,
            &self.raster_fill_entry,
            &self.sides_entry,
            &self.gear_module_entry,
            &self.gear_teeth_entry,
            &self.gear_pressure_angle_entry,
            &self.sprocket_pitch_entry,
            &self.sprocket_teeth_entry,
            &self.sprocket_roller_diameter_entry,
            &self.offset_entry,
            &self.fillet_entry,
            &self.chamfer_entry,
        ];

        for entry in entries {
            let focus_controller = EventControllerFocus::new();
            let has_focus_enter = self.has_focus.clone();
            focus_controller.connect_enter(move |_| {
                *has_focus_enter.borrow_mut() = true;
            });

            let has_focus_leave = self.has_focus.clone();
            focus_controller.connect_leave(move |_| {
                *has_focus_leave.borrow_mut() = false;
            });

            entry.add_controller(focus_controller);
        }

        // Track focus for text entry (content)
        let focus_controller = EventControllerFocus::new();
        let has_focus_enter = self.has_focus.clone();
        focus_controller.connect_enter(move |_| {
            *has_focus_enter.borrow_mut() = true;
        });

        let has_focus_leave = self.has_focus.clone();
        focus_controller.connect_leave(move |_| {
            *has_focus_leave.borrow_mut() = false;
        });
        self.text_entry.add_controller(focus_controller);
    }

    /// Clear the focus flag - call this when user interacts with the canvas
    pub fn clear_focus(&self) {
        *self.has_focus.borrow_mut() = false;
    }
}

// UI Section builders
impl PropertiesPanel {
    fn build_position_section() -> (Frame, Entry, Entry, Label, Label) {
        let frame = Self::create_section(&t!("Position"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let x_label = Label::new(Some(&t!("X:")));
        x_label.set_halign(gtk4::Align::Start);
        let pos_x_entry = Entry::new();
        pos_x_entry.set_hexpand(true);
        let x_unit_label = Label::new(Some("mm"));
        x_unit_label.set_width_chars(4);
        x_unit_label.set_halign(gtk4::Align::End);
        x_unit_label.set_xalign(1.0);

        let y_label = Label::new(Some(&t!("Y:")));
        y_label.set_halign(gtk4::Align::Start);
        let pos_y_entry = Entry::new();
        pos_y_entry.set_hexpand(true);
        let y_unit_label = Label::new(Some("mm"));
        y_unit_label.set_width_chars(4);
        y_unit_label.set_halign(gtk4::Align::End);
        y_unit_label.set_xalign(1.0);

        grid.attach(&x_label, 0, 0, 1, 1);
        grid.attach(&pos_x_entry, 1, 0, 1, 1);
        grid.attach(&x_unit_label, 2, 0, 1, 1);
        grid.attach(&y_label, 0, 1, 1, 1);
        grid.attach(&pos_y_entry, 1, 1, 1, 1);
        grid.attach(&y_unit_label, 2, 1, 1, 1);

        frame.set_child(Some(&grid));
        (frame, pos_x_entry, pos_y_entry, x_unit_label, y_unit_label)
    }

    fn build_size_section() -> (Frame, Entry, Entry, CheckButton, Label, Label) {
        let frame = Self::create_section(&t!("Size"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let width_label = Label::new(Some(&t!("Width:")));
        width_label.set_halign(gtk4::Align::Start);
        let width_entry = Entry::new();
        width_entry.set_hexpand(true);
        let width_unit_label = Label::new(Some("mm"));
        width_unit_label.set_width_chars(4);
        width_unit_label.set_halign(gtk4::Align::End);
        width_unit_label.set_xalign(1.0);

        let height_label = Label::new(Some(&t!("Height:")));
        height_label.set_halign(gtk4::Align::Start);
        let height_entry = Entry::new();
        height_entry.set_hexpand(true);
        let height_unit_label = Label::new(Some("mm"));
        height_unit_label.set_width_chars(4);
        height_unit_label.set_halign(gtk4::Align::End);
        height_unit_label.set_xalign(1.0);

        let lock_aspect_label = Label::new(Some(&t!("Lock Aspect:")));
        lock_aspect_label.set_halign(gtk4::Align::Start);
        let lock_aspect_ratio = CheckButton::new();
        lock_aspect_ratio.set_active(true);

        grid.attach(&width_label, 0, 0, 1, 1);
        grid.attach(&width_entry, 1, 0, 1, 1);
        grid.attach(&width_unit_label, 2, 0, 1, 1);
        grid.attach(&height_label, 0, 1, 1, 1);
        grid.attach(&height_entry, 1, 1, 1, 1);
        grid.attach(&height_unit_label, 2, 1, 1, 1);
        grid.attach(&lock_aspect_label, 0, 2, 1, 1);
        grid.attach(&lock_aspect_ratio, 1, 2, 1, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            width_entry,
            height_entry,
            lock_aspect_ratio,
            width_unit_label,
            height_unit_label,
        )
    }

    fn build_rotation_section() -> (Frame, Entry) {
        let frame = Self::create_section(&t!("Rotation"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let rot_label = Label::new(Some(&t!("Angle:")));
        rot_label.set_halign(gtk4::Align::Start);
        let rotation_entry = Entry::new();
        rotation_entry.set_hexpand(true);
        let rot_unit = Label::new(Some("deg"));

        grid.attach(&rot_label, 0, 0, 1, 1);
        grid.attach(&rotation_entry, 1, 0, 1, 1);
        grid.attach(&rot_unit, 2, 0, 1, 1);

        frame.set_child(Some(&grid));
        (frame, rotation_entry)
    }

    fn build_corner_section() -> (Frame, Entry, CheckButton, Label) {
        let frame = Self::create_section(&t!("Corner"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let radius_label = Label::new(Some(&t!("Radius:")));
        radius_label.set_halign(gtk4::Align::Start);
        let corner_radius_entry = Entry::new();
        corner_radius_entry.set_hexpand(true);
        let radius_unit_label = Label::new(Some("mm"));
        radius_unit_label.set_width_chars(4);
        radius_unit_label.set_halign(gtk4::Align::End);
        radius_unit_label.set_xalign(1.0);

        let slot_label = Label::new(Some(&t!("Slot Mode:")));
        slot_label.set_halign(gtk4::Align::Start);
        let is_slot_check = CheckButton::new();

        grid.attach(&radius_label, 0, 0, 1, 1);
        grid.attach(&corner_radius_entry, 1, 0, 1, 1);
        grid.attach(&radius_unit_label, 2, 0, 1, 1);
        grid.attach(&slot_label, 0, 1, 1, 1);
        grid.attach(&is_slot_check, 1, 1, 1, 1);

        frame.set_child(Some(&grid));
        (frame, corner_radius_entry, is_slot_check, radius_unit_label)
    }

    fn build_text_section() -> (
        Frame,
        Entry,
        DropDown,
        CheckButton,
        CheckButton,
        Entry,
        Label,
    ) {
        let frame = Self::create_section(&t!("Text"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let text_content_label = Label::new(Some(&t!("Content:")));
        text_content_label.set_halign(gtk4::Align::Start);
        let text_entry = Entry::new();
        text_entry.set_hexpand(true);

        let font_label = Label::new(Some(&t!("Font:")));
        font_label.set_halign(gtk4::Align::Start);
        let font_model = StringList::new(&[]);
        font_model.append("Sans");
        for fam in font_manager::list_font_families() {
            if fam != "Sans" {
                font_model.append(&fam);
            }
        }
        let font_family_combo = DropDown::new(Some(font_model), None::<Expression>);
        font_family_combo.set_hexpand(true);

        let style_label = Label::new(Some(&t!("Style:")));
        style_label.set_halign(gtk4::Align::Start);
        let font_bold_check = CheckButton::with_label(&t!("Bold"));
        let font_italic_check = CheckButton::with_label(&t!("Italic"));
        let style_box = Box::new(Orientation::Horizontal, 8);
        style_box.append(&font_bold_check);
        style_box.append(&font_italic_check);

        let font_size_label = Label::new(Some(&t!("Size:")));
        font_size_label.set_halign(gtk4::Align::Start);
        let font_size_entry = Entry::new();
        font_size_entry.set_hexpand(true);
        let font_size_unit_label = Label::new(Some("pt"));
        font_size_unit_label.set_width_chars(4);
        font_size_unit_label.set_halign(gtk4::Align::End);
        font_size_unit_label.set_xalign(1.0);

        grid.attach(&text_content_label, 0, 0, 1, 1);
        grid.attach(&text_entry, 1, 0, 2, 1);
        grid.attach(&font_label, 0, 1, 1, 1);
        grid.attach(&font_family_combo, 1, 1, 2, 1);
        grid.attach(&style_label, 0, 2, 1, 1);
        grid.attach(&style_box, 1, 2, 2, 1);
        grid.attach(&font_size_label, 0, 3, 1, 1);
        grid.attach(&font_size_entry, 1, 3, 1, 1);
        grid.attach(&font_size_unit_label, 2, 3, 1, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            text_entry,
            font_family_combo,
            font_bold_check,
            font_italic_check,
            font_size_entry,
            font_size_unit_label,
        )
    }

    fn build_polygon_section() -> (Frame, Entry) {
        let frame = Self::create_section(&t!("Polygon"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let sides_label = Label::new(Some(&t!("Sides:")));
        sides_label.set_halign(gtk4::Align::Start);
        let sides_entry = Entry::new();
        sides_entry.set_hexpand(true);

        grid.attach(&sides_label, 0, 0, 1, 1);
        grid.attach(&sides_entry, 1, 0, 1, 1);

        frame.set_child(Some(&grid));
        (frame, sides_entry)
    }

    fn build_gear_section() -> (Frame, Entry, Entry, Entry) {
        let frame = Self::create_section(&t!("Gear"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let module_label = Label::new(Some(&t!("Module:")));
        module_label.set_halign(gtk4::Align::Start);
        let gear_module_entry = Entry::new();
        gear_module_entry.set_hexpand(true);

        let teeth_label = Label::new(Some(&t!("Teeth:")));
        teeth_label.set_halign(gtk4::Align::Start);
        let gear_teeth_entry = Entry::new();
        gear_teeth_entry.set_hexpand(true);

        let pa_label = Label::new(Some(&t!("Pressure Angle:")));
        pa_label.set_halign(gtk4::Align::Start);
        let gear_pressure_angle_entry = Entry::new();
        gear_pressure_angle_entry.set_hexpand(true);

        grid.attach(&module_label, 0, 0, 1, 1);
        grid.attach(&gear_module_entry, 1, 0, 1, 1);
        grid.attach(&teeth_label, 0, 1, 1, 1);
        grid.attach(&gear_teeth_entry, 1, 1, 1, 1);
        grid.attach(&pa_label, 0, 2, 1, 1);
        grid.attach(&gear_pressure_angle_entry, 1, 2, 1, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            gear_module_entry,
            gear_teeth_entry,
            gear_pressure_angle_entry,
        )
    }

    fn build_sprocket_section() -> (Frame, Entry, Entry, Entry) {
        let frame = Self::create_section(&t!("Sprocket"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let pitch_label = Label::new(Some(&t!("Pitch:")));
        pitch_label.set_halign(gtk4::Align::Start);
        let sprocket_pitch_entry = Entry::new();
        sprocket_pitch_entry.set_hexpand(true);

        let teeth_label = Label::new(Some(&t!("Teeth:")));
        teeth_label.set_halign(gtk4::Align::Start);
        let sprocket_teeth_entry = Entry::new();
        sprocket_teeth_entry.set_hexpand(true);

        let roller_label = Label::new(Some(&t!("Roller Dia:")));
        roller_label.set_halign(gtk4::Align::Start);
        let sprocket_roller_diameter_entry = Entry::new();
        sprocket_roller_diameter_entry.set_hexpand(true);

        grid.attach(&pitch_label, 0, 0, 1, 1);
        grid.attach(&sprocket_pitch_entry, 1, 0, 1, 1);
        grid.attach(&teeth_label, 0, 1, 1, 1);
        grid.attach(&sprocket_teeth_entry, 1, 1, 1, 1);
        grid.attach(&roller_label, 0, 2, 1, 1);
        grid.attach(&sprocket_roller_diameter_entry, 1, 2, 1, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            sprocket_pitch_entry,
            sprocket_teeth_entry,
            sprocket_roller_diameter_entry,
        )
    }

    fn build_geometry_ops_section() -> (Frame, Entry, Entry, Entry, Label, Label, Label) {
        let frame = Self::create_section(&t!("Geometry Operations"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let offset_label = Label::new(Some(&t!("Offset:")));
        offset_label.set_halign(gtk4::Align::Start);
        let offset_entry = Entry::new();
        offset_entry.set_text("1.0");
        offset_entry.set_hexpand(true);
        let offset_unit_label = Label::new(Some("mm"));

        let fillet_label = Label::new(Some(&t!("Fillet:")));
        fillet_label.set_halign(gtk4::Align::Start);
        let fillet_entry = Entry::new();
        fillet_entry.set_text("2.0");
        fillet_entry.set_hexpand(true);
        let fillet_unit_label = Label::new(Some("mm"));

        let chamfer_label = Label::new(Some(&t!("Chamfer:")));
        chamfer_label.set_halign(gtk4::Align::Start);
        let chamfer_entry = Entry::new();
        chamfer_entry.set_text("2.0");
        chamfer_entry.set_hexpand(true);
        let chamfer_unit_label = Label::new(Some("mm"));

        grid.attach(&offset_label, 0, 0, 1, 1);
        grid.attach(&offset_entry, 1, 0, 1, 1);
        grid.attach(&offset_unit_label, 2, 0, 1, 1);
        grid.attach(&fillet_label, 0, 1, 1, 1);
        grid.attach(&fillet_entry, 1, 1, 1, 1);
        grid.attach(&fillet_unit_label, 2, 1, 1, 1);
        grid.attach(&chamfer_label, 0, 2, 1, 1);
        grid.attach(&chamfer_entry, 1, 2, 1, 1);
        grid.attach(&chamfer_unit_label, 2, 2, 1, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            offset_entry,
            fillet_entry,
            chamfer_entry,
            offset_unit_label,
            fillet_unit_label,
            chamfer_unit_label,
        )
    }

    fn build_cam_section() -> (
        Frame,
        DropDown,
        Entry,
        Entry,
        Entry,
        Entry,
        DropDown,
        Entry,
        Label,
        Label,
        Label,
    ) {
        let frame = Self::create_section(&t!("CAM Properties"));
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Operation Type
        let op_label = Label::new(Some(&t!("Operation:")));
        op_label.set_halign(gtk4::Align::Start);
        let op_model = StringList::new(&[]);
        op_model.append(&t!("Profile"));
        op_model.append(&t!("Pocket"));
        let op_type_combo = DropDown::new(Some(op_model), None::<Expression>);
        op_type_combo.set_hexpand(true);

        // Pocket Depth
        let depth_label = Label::new(Some(&t!("Depth:")));
        depth_label.set_halign(gtk4::Align::Start);
        let depth_entry = Entry::new();
        depth_entry.set_hexpand(true);
        let depth_unit_label = Label::new(Some("mm"));
        depth_unit_label.set_width_chars(4);
        depth_unit_label.set_halign(gtk4::Align::End);
        depth_unit_label.set_xalign(1.0);

        // Step Down
        let step_down_label = Label::new(Some(&t!("Step Down:")));
        step_down_label.set_halign(gtk4::Align::Start);
        let step_down_entry = Entry::new();
        step_down_entry.set_hexpand(true);
        let step_down_unit_label = Label::new(Some("mm"));
        step_down_unit_label.set_width_chars(4);
        step_down_unit_label.set_halign(gtk4::Align::End);
        step_down_unit_label.set_xalign(1.0);

        // Step In (for pockets)
        let step_in_label = Label::new(Some(&t!("Step In:")));
        step_in_label.set_halign(gtk4::Align::Start);
        let step_in_entry = Entry::new();
        step_in_entry.set_hexpand(true);
        let step_in_unit_label = Label::new(Some("mm"));
        step_in_unit_label.set_width_chars(4);
        step_in_unit_label.set_halign(gtk4::Align::End);
        step_in_unit_label.set_xalign(1.0);

        // Ramp Angle
        let ramp_angle_label = Label::new(Some(&t!("Ramp Angle:")));
        ramp_angle_label.set_halign(gtk4::Align::Start);
        let ramp_angle_entry = Entry::new();
        ramp_angle_entry.set_hexpand(true);
        let ramp_angle_unit_label = Label::new(Some("deg"));
        ramp_angle_unit_label.set_width_chars(4);
        ramp_angle_unit_label.set_halign(gtk4::Align::End);
        ramp_angle_unit_label.set_xalign(1.0);

        // Pocket Strategy
        let strategy_label = Label::new(Some(&t!("Strategy:")));
        strategy_label.set_halign(gtk4::Align::Start);
        let strategy_model = StringList::new(&[]);
        strategy_model.append(&t!("Raster"));
        strategy_model.append(&t!("Offset"));
        strategy_model.append(&t!("Adaptive"));
        let strategy_combo = DropDown::new(Some(strategy_model), None::<Expression>);
        strategy_combo.set_hexpand(true);

        // Raster Fill (inverse inset)
        let raster_fill_label = Label::new(Some(&t!("Raster Fill (%):")));
        raster_fill_label.set_halign(gtk4::Align::Start);
        let raster_fill_entry = Entry::new();
        raster_fill_entry.set_hexpand(true);
        let raster_fill_hint = Label::new(Some("0 = no raster, 100 = full length"));
        raster_fill_hint.add_css_class("dim-label");
        raster_fill_hint.set_halign(gtk4::Align::Start);

        grid.attach(&op_label, 0, 0, 1, 1);
        grid.attach(&op_type_combo, 1, 0, 1, 1);
        grid.attach(&depth_label, 0, 1, 1, 1);
        grid.attach(&depth_entry, 1, 1, 1, 1);
        grid.attach(&depth_unit_label, 2, 1, 1, 1);
        grid.attach(&step_down_label, 0, 2, 1, 1);
        grid.attach(&step_down_entry, 1, 2, 1, 1);
        grid.attach(&step_down_unit_label, 2, 2, 1, 1);
        grid.attach(&step_in_label, 0, 3, 1, 1);
        grid.attach(&step_in_entry, 1, 3, 1, 1);
        grid.attach(&step_in_unit_label, 2, 3, 1, 1);
        grid.attach(&ramp_angle_label, 0, 4, 1, 1);
        grid.attach(&ramp_angle_entry, 1, 4, 1, 1);
        grid.attach(&ramp_angle_unit_label, 2, 4, 1, 1);
        grid.attach(&strategy_label, 0, 5, 1, 1);
        grid.attach(&strategy_combo, 1, 5, 1, 1);
        grid.attach(&raster_fill_label, 0, 6, 1, 1);
        grid.attach(&raster_fill_entry, 1, 6, 1, 1);
        grid.attach(&raster_fill_hint, 0, 7, 3, 1);

        frame.set_child(Some(&grid));
        (
            frame,
            op_type_combo,
            depth_entry,
            step_down_entry,
            step_in_entry,
            ramp_angle_entry,
            strategy_combo,
            raster_fill_entry,
            depth_unit_label,
            step_down_unit_label,
            step_in_unit_label,
        )
    }
}
