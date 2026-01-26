//! Designer View - Main designer UI container
//!
//! This module provides the DesignerView which contains the main designer interface
//! including toolbox, canvas, properties panel, and layers panel.

use crate::t;
use crate::ui::gtk::designer_canvas::DesignerCanvas;
use crate::ui::gtk::designer_layers::LayersPanel;
use crate::ui::gtk::designer_properties::PropertiesPanel;
use crate::ui::gtk::designer_toolbox::{DesignerTool, DesignerToolbox};
use crate::ui::gtk::osd_format::format_zoom_center_cursor;
use gcodekit5_designer::designer_state::DesignerState;
use gcodekit5_designer::model::{DesignerShape, Shape};
use gcodekit5_designer::serialization::DesignFile;
use gcodekit5_designer::stock_removal::StockMaterial;
use gcodekit5_devicedb::DeviceManager;
use gcodekit5_settings::controller::SettingsController;
use gtk4::gdk::{Key, ModifierType};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box, EventControllerKey, FileChooserAction, FileChooserNative, GestureClick, Grid,
    Label, Orientation, Overlay, Paned, Popover, ResponseType, Scrollbar,
};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::error;

pub struct DesignerView {
    pub widget: Box,
    canvas: Rc<DesignerCanvas>,
    toolbox: Rc<DesignerToolbox>,
    _properties: Rc<PropertiesPanel>,
    layers: Rc<LayersPanel>,
    status_label: Label,
    _coord_label: Label,
    current_file: Rc<RefCell<Option<PathBuf>>>,
    on_gcode_generated: Rc<RefCell<Option<std::boxed::Box<dyn Fn(String)>>>>,
    settings_persistence: Option<Rc<RefCell<gcodekit5_settings::SettingsPersistence>>>,
}

impl DesignerView {
    pub fn new(
        device_manager: Option<Arc<DeviceManager>>,
        settings_controller: Rc<SettingsController>,
        status_bar: Option<crate::ui::gtk::status_bar::StatusBar>,
    ) -> Rc<Self> {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Create designer state
        let state = Rc::new(RefCell::new(DesignerState::new()));

        // Create main horizontal layout (toolbox + canvas + properties)
        let main_box = Box::new(Orientation::Horizontal, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        // Create toolbox + left sidebar container (toolbox + view/legend)
        let toolbox = DesignerToolbox::new(state.clone(), settings_controller.clone());
        let left_sidebar = Box::new(Orientation::Vertical, 0);
        left_sidebar.set_vexpand(true);
        left_sidebar.set_hexpand(false);
        left_sidebar.set_halign(gtk4::Align::Fill);
        left_sidebar.append(&toolbox.widget);

        // Keep left sidebar at ~20% of the main window width (set on first map).
        let last_left_width = Rc::new(std::cell::Cell::new(-1));
        {
            let left_sidebar = left_sidebar.clone();
            let last_left_width = last_left_width.clone();
            let container_width = container.clone();
            container.connect_map(move |_| {
                let left_sidebar = left_sidebar.clone();
                let last_left_width = last_left_width.clone();
                let container_width = container_width.clone();
                gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                    let w = container_width.width();
                    if w <= 0 {
                        return gtk4::glib::ControlFlow::Continue;
                    }
                    let target = ((w as f64) * 0.20).round() as i32;
                    let target = target.max(160);
                    if last_left_width.get() != target {
                        last_left_width.set(target);
                        left_sidebar.set_width_request(target);
                    }
                    gtk4::glib::ControlFlow::Break
                });
            });
        }

        // Paned layout: left sidebar is resizable
        let left_paned = Paned::new(Orientation::Horizontal);
        left_paned.set_start_child(Some(&left_sidebar));
        left_paned.set_resize_start_child(true);
        left_paned.set_shrink_start_child(false);

        // Create canvas
        let canvas = DesignerCanvas::new(
            state.clone(),
            Some(toolbox.clone()),
            device_manager.clone(),
            status_bar.clone(),
            Some(settings_controller.clone()),
        );

        // Create Grid for Canvas + Scrollbars
        let canvas_grid = Grid::new();
        canvas_grid.set_hexpand(true);
        canvas_grid.set_vexpand(true);

        canvas.widget.set_hexpand(true);
        canvas.widget.set_vexpand(true);

        // Overlay for floating controls
        let overlay = Overlay::new();
        overlay.set_child(Some(&canvas.widget));

        // Floating Controls (Bottom Right)
        let (
            floating_box,
            float_zoom_in,
            float_zoom_out,
            float_fit,
            float_reset,
            float_fit_device,
            scrollbars_btn,
        ) = Self::create_floating_controls(device_manager.is_some());

        // Empty state (shown when no shapes)
        let (
            empty_box,
            empty_new_btn,
            empty_open_btn,
            empty_import_svg_btn,
            empty_import_dxf_btn,
            empty_import_stl_btn,
        ) = Self::create_empty_state(&settings_controller);

        overlay.add_overlay(&empty_box);
        overlay.add_overlay(&floating_box);

        // Status Panel (Bottom Left)
        let (status_box, status_label_osd, units_badge) = Self::create_status_panel();
        overlay.add_overlay(&status_box);

        // Attach Overlay to Grid (instead of direct canvas)
        canvas_grid.attach(&overlay, 0, 0, 1, 1);

        // Scrollbars
        // Range: use shared world extent (±WORLD_EXTENT_MM)
        let world_extent = gcodekit5_core::constants::WORLD_EXTENT_MM as f64;
        let h_adjustment = Adjustment::new(0.0, -world_extent, world_extent, 10.0, 100.0, 100.0);
        let v_adjustment = Adjustment::new(0.0, -world_extent, world_extent, 10.0, 100.0, 100.0);

        let h_scrollbar = Scrollbar::new(Orientation::Horizontal, Some(&h_adjustment));
        let v_scrollbar = Scrollbar::new(Orientation::Vertical, Some(&v_adjustment));

        // Default hidden (toggleable) to maximize canvas space
        h_scrollbar.set_visible(false);
        v_scrollbar.set_visible(false);

        canvas_grid.attach(&v_scrollbar, 1, 0, 1, 1);
        canvas_grid.attach(&h_scrollbar, 0, 1, 1, 1);

        // Set center area placeholder to be replaced later
        // We'll update this after creating the center paned
        left_paned.set_resize_end_child(true);
        left_paned.set_shrink_end_child(false);
        left_paned.set_hexpand(true);
        left_paned.set_vexpand(true);
        left_paned.set_position(180); // Initial position for left sidebar

        main_box.append(&left_paned);

        // Connect scrollbars to canvas pan
        let canvas_h = canvas.clone();
        h_adjustment.connect_value_changed(move |adj| {
            let val = adj.value();
            let mut state = canvas_h.state.borrow_mut();
            // Pan is opposite to scroll value usually
            let current_pan_y = state.canvas.pan_y();
            state.canvas.set_pan(-val, current_pan_y);
            drop(state);
            canvas_h.widget.queue_draw();
        });

        let canvas_v = canvas.clone();
        v_adjustment.connect_value_changed(move |adj| {
            let val = adj.value();
            let mut state = canvas_v.state.borrow_mut();
            // Positive scroll value (down) moves content up (positive pan_y)
            let current_pan_x = state.canvas.pan_x();
            state.canvas.set_pan(current_pan_x, val);
            drop(state);
            canvas_v.widget.queue_draw();
        });

        // Pass adjustments to canvas
        canvas.set_adjustments(h_adjustment.clone(), v_adjustment.clone());

        // Connect Floating Zoom Buttons
        let canvas_zoom = canvas.clone();
        float_zoom_in.connect_clicked(move |_| {
            canvas_zoom.zoom_in();
        });

        let canvas_zoom = canvas.clone();
        float_zoom_out.connect_clicked(move |_| {
            canvas_zoom.zoom_out();
        });

        let canvas_zoom = canvas.clone();
        float_fit.connect_clicked(move |_| {
            canvas_zoom.zoom_fit();
        });

        let canvas_reset = canvas.clone();
        float_reset.connect_clicked(move |_| {
            canvas_reset.reset_view();
        });

        let canvas_fitdev = canvas.clone();
        float_fit_device.connect_clicked(move |_| {
            canvas_fitdev.fit_to_device_area();
            canvas_fitdev.widget.queue_draw();
        });

        // Scrollbars toggle
        let show_scrollbars = Rc::new(std::cell::Cell::new(false));
        let show_scrollbars_btn = show_scrollbars.clone();
        let hsb = h_scrollbar.clone();
        let vsb = v_scrollbar.clone();
        scrollbars_btn.connect_clicked(move |_| {
            let next = !show_scrollbars_btn.get();
            show_scrollbars_btn.set(next);
            hsb.set_visible(next);
            vsb.set_visible(next);
        });

        // Auto-fit when designer is mapped (visible) — schedule after layout like Visualizer
        let canvas_map = canvas.clone();
        container.connect_map(move |_| {
            let canvas_run = canvas_map.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                // Ensure viewport knows the correct size before fitting
                let width = canvas_run.widget.width() as f64;
                let height = canvas_run.widget.height() as f64;
                if width > 0.0 && height > 0.0 {
                    if let Ok(mut state) = canvas_run.state.try_borrow_mut() {
                        state.canvas.set_canvas_size(width, height);
                    }
                }

                // Always fit to device on initialization as per user request
                canvas_run.fit_to_device_area();
                canvas_run.widget.queue_draw();
                gtk4::glib::ControlFlow::Break
            });
        });

        // Create right sidebar with properties and layers
        let right_sidebar = Box::new(Orientation::Vertical, 5);
        right_sidebar.set_hexpand(false);
        right_sidebar.set_halign(gtk4::Align::End);

        // Keep right sidebar at ~20% of the main window width (set on first map).
        let last_right_width = Rc::new(std::cell::Cell::new(-1));
        {
            let right_sidebar = right_sidebar.clone();
            let last_right_width = last_right_width.clone();
            let container_width = container.clone();
            container.connect_map(move |_| {
                let right_sidebar = right_sidebar.clone();
                let last_right_width = last_right_width.clone();
                let container_width = container_width.clone();
                gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                    let w = container_width.width();
                    if w <= 0 {
                        return gtk4::glib::ControlFlow::Continue;
                    }
                    let target = ((w as f64) * 0.20).round() as i32;
                    let target = target.clamp(240, 520);
                    if last_right_width.get() != target {
                        last_right_width.set(target);
                        right_sidebar.set_width_request(target);
                    }
                    gtk4::glib::ControlFlow::Break
                });
            });
        }

        // Create properties panel
        let properties = PropertiesPanel::new(
            state.clone(),
            settings_controller.persistence.clone(),
            canvas.preview_shapes.clone(),
        );
        properties.widget.set_vexpand(true);
        properties.widget.set_valign(gtk4::Align::Fill);

        // Set up redraw callback for properties
        let canvas_redraw = canvas.clone();
        properties.set_redraw_callback(move || {
            let show_toolpaths = canvas_redraw.state.borrow().show_toolpaths;
            if show_toolpaths {
                canvas_redraw.generate_preview_toolpaths();
            }
            canvas_redraw.widget.queue_draw();
        });

        // Inspector header + hide button (matches DeviceConsole / Visualizer sidebar UX)
        let props_hidden = Rc::new(Cell::new(false));

        let inspector_header = Box::new(Orientation::Horizontal, 6);
        inspector_header.set_margin_start(6);
        inspector_header.set_margin_end(6);
        inspector_header.set_margin_top(6);

        let inspector_label = Label::builder()
            .label(t!("Inspector"))
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .build();
        inspector_label.set_hexpand(true);
        inspector_header.append(&inspector_label);

        let props_hide_btn = gtk4::Button::builder()
            .tooltip_text(t!("Hide Properties"))
            .build();
        props_hide_btn
            .update_property(&[gtk4::accessible::Property::Label(&t!("Hide Properties"))]);
        {
            let child = Box::new(Orientation::Horizontal, 6);
            child.append(&gtk4::Image::from_icon_name("view-conceal-symbolic"));
            child.append(&Label::new(Some(&t!("Hide"))));
            props_hide_btn.set_child(Some(&child));
        }
        inspector_header.append(&props_hide_btn);

        right_sidebar.append(&inspector_header);

        // Create layers panel
        let layers = Rc::new(LayersPanel::new(state.clone(), canvas.widget.clone()));
        layers.widget.set_vexpand(true);
        layers.widget.set_valign(gtk4::Align::Fill);

        // Draggable divider between Properties and Layers
        let inspector_paned = Paned::new(Orientation::Vertical);
        inspector_paned.set_vexpand(true);
        inspector_paned.set_start_child(Some(&properties.widget));
        inspector_paned.set_end_child(Some(&layers.widget));
        inspector_paned.set_resize_start_child(true);
        inspector_paned.set_resize_end_child(true);
        inspector_paned.set_shrink_start_child(false);
        inspector_paned.set_shrink_end_child(false);
        inspector_paned.set_position(520);

        right_sidebar.append(&inspector_paned);

        // Floating unhide button (top-right of canvas)
        let props_show_btn = gtk4::Button::builder()
            .tooltip_text(t!("Unhide Properties"))
            .build();
        props_show_btn
            .update_property(&[gtk4::accessible::Property::Label(&t!("Unhide Properties"))]);
        {
            let child = Box::new(Orientation::Horizontal, 6);
            child.append(&gtk4::Image::from_icon_name("view-reveal-symbolic"));
            child.append(&Label::new(Some(&t!("Unhide"))));
            props_show_btn.set_child(Some(&child));
        }

        let props_show_panel = Box::new(Orientation::Horizontal, 0);
        props_show_panel.add_css_class("visualizer-osd");
        props_show_panel.add_css_class("osd-controls");
        props_show_panel.set_halign(gtk4::Align::End);
        props_show_panel.set_valign(gtk4::Align::Start);
        props_show_panel.set_margin_end(12);
        props_show_panel.set_margin_top(12);
        props_show_panel.append(&props_show_btn);
        props_show_panel.set_visible(false);
        overlay.add_overlay(&props_show_panel);

        {
            let right_sidebar = right_sidebar.clone();
            let props_hidden = props_hidden.clone();
            let show_panel = props_show_panel.clone();
            let hide_btn = props_hide_btn.clone();
            props_hide_btn.connect_clicked(move |_| {
                if props_hidden.get() {
                    return;
                }
                right_sidebar.set_visible(false);
                hide_btn.set_sensitive(false);
                props_hidden.set(true);
                show_panel.set_visible(true);
            });
        }

        {
            let right_sidebar = right_sidebar.clone();
            let props_hidden = props_hidden.clone();
            let show_panel = props_show_panel.clone();
            let hide_btn = props_hide_btn.clone();
            props_show_btn.connect_clicked(move |_| {
                if !props_hidden.get() {
                    return;
                }
                right_sidebar.set_visible(true);
                hide_btn.set_sensitive(true);
                props_hidden.set(false);
                show_panel.set_visible(false);
            });
        }

        // Legend moved to left sidebar

        // Give canvas references to panels
        canvas.set_properties_panel(properties.clone());
        canvas.set_layers_panel(layers.clone());

        // Paned layout: right sidebar is resizable from the center paned
        let center_paned = Paned::new(Orientation::Horizontal);
        center_paned.set_start_child(Some(&canvas_grid));
        center_paned.set_end_child(Some(&right_sidebar));
        center_paned.set_resize_start_child(true);
        center_paned.set_resize_end_child(true);
        center_paned.set_shrink_start_child(false);
        center_paned.set_shrink_end_child(false);
        center_paned.set_hexpand(true);
        center_paned.set_vexpand(true);
        center_paned.set_position(600); // Will be adjusted on map

        // Now set the center paned as the end child of the left paned
        left_paned.set_end_child(Some(&center_paned));

        // Auto-size the center paned position when window is mapped
        let center_paned_size = center_paned.clone();
        let right_sidebar_width = right_sidebar.clone();
        container.connect_map(move |_cont| {
            let center_paned = center_paned_size.clone();
            let right_sidebar = right_sidebar_width.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                let total_width = center_paned.width();
                if total_width > 0 {
                    // Get the right sidebar preferred width (should be set by now)
                    let right_w = right_sidebar.width_request();
                    let right_w = if right_w > 0 { right_w } else { 300 };
                    // Position center_paned divider to give most space to canvas
                    let canvas_width = total_width - right_w;
                    if canvas_width > 100 {
                        center_paned.set_position(canvas_width);
                    }
                }
                gtk4::glib::ControlFlow::Break
            });
        });

        container.append(&main_box);

        // Hidden labels retained for internal status messages (status bar removed)
        let status_label = Label::new(None);
        let coord_label = Label::new(None);

        // View controls (moved to helper function)
        let view_controls_expander =
            Self::create_view_controls_expander(&state, &canvas, &settings_controller);
        left_sidebar.append(&view_controls_expander);

        // Start status OSD update loop
        Self::start_status_update_loop(
            status_label_osd,
            units_badge,
            empty_box.clone(),
            canvas.clone(),
            settings_controller.clone(),
        );

        let current_file = Rc::new(RefCell::new(None));
        let on_gcode_generated: Rc<RefCell<Option<std::boxed::Box<dyn Fn(String)>>>> =
            Rc::new(RefCell::new(None));

        // Connect Generate G-Code button
        let canvas_gen = canvas.clone();
        let on_gen = on_gcode_generated.clone();
        let status_label_gen = status_label.clone();

        toolbox.connect_generate_clicked(move || {
            let mut state = canvas_gen.state.borrow_mut();

            // Copy settings to avoid borrow issues
            let feed_rate = state.tool_settings.feed_rate;
            let spindle_speed = state.tool_settings.spindle_speed;
            let tool_diameter = state.tool_settings.tool_diameter;
            let cut_depth = state.tool_settings.cut_depth;
            let start_depth = state.tool_settings.start_depth;

            // Update toolpath generator settings from state
            state.toolpath_generator.set_feed_rate(feed_rate);
            state.toolpath_generator.set_spindle_speed(spindle_speed);
            state.toolpath_generator.set_tool_diameter(tool_diameter);
            state.toolpath_generator.set_cut_depth(cut_depth);
            state.toolpath_generator.set_start_depth(start_depth);
            state.toolpath_generator.set_step_in(tool_diameter * 0.4); // Default stepover

            let gcode = state.generate_gcode();
            drop(state);

            status_label_gen.set_text(&t!("G-Code generated"));

            if let Some(callback) = on_gen.borrow().as_ref() {
                callback(gcode);
            }
        });

        // Connect fast shape gallery to insert shapes
        let canvas_shape = canvas.clone();
        let layers_shape = layers.clone();
        toolbox
            .fast_shape_gallery()
            .connect_shape_selected(move |shape| {
                let mut state = canvas_shape.state.borrow_mut();
                state.add_shape_with_undo(shape);
                drop(state);

                // Refresh layers panel
                layers_shape.refresh(&canvas_shape.state);
                canvas_shape.widget.queue_draw();
            });

        let view = Rc::new(Self {
            widget: container,
            canvas: canvas.clone(),
            toolbox: toolbox.clone(),
            _properties: properties.clone(),
            layers: layers.clone(),
            status_label,
            _coord_label: coord_label,
            current_file,
            on_gcode_generated,
            settings_persistence: Some(settings_controller.persistence.clone()),
        });

        // Empty state actions
        {
            let v = view.clone();
            empty_new_btn.connect_clicked(move |_| v.new_file());
        }
        {
            let v = view.clone();
            empty_open_btn.connect_clicked(move |_| v.open_file());
        }
        {
            let v = view.clone();
            empty_import_svg_btn.connect_clicked(move |_| v.import_svg_file());
        }
        {
            let v = view.clone();
            empty_import_dxf_btn.connect_clicked(move |_| v.import_dxf_file());
        }
        {
            let v = view.clone();
            empty_import_stl_btn.connect_clicked(move |_| v.import_stl_file());
        }

        // Add settings change listener for STL import feature
        {
            let empty_import_stl_btn = empty_import_stl_btn.clone();
            settings_controller.on_setting_changed(move |key, value| {
                if key != "enable_stl_import" {
                    return;
                }
                let enabled = value == "true";
                empty_import_stl_btn.set_visible(enabled);
            });
        }

        // Update properties panel and toolbox when selection changes
        let props_update = properties.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Check if we need to update properties (when canvas is redrawn or selection changes)
            props_update.update_from_selection();

            gtk4::glib::ControlFlow::Continue
        });

        // Setup keyboard shortcuts
        Self::setup_keyboard_shortcuts(&canvas);

        // Grab focus initially
        canvas.widget.grab_focus();

        view
    }

    /// Creates the view controls expander for the left sidebar
    fn create_view_controls_expander(
        state: &Rc<RefCell<DesignerState>>,
        canvas: &Rc<DesignerCanvas>,
        settings_controller: &Rc<SettingsController>,
    ) -> gtk4::Expander {
        let view_controls_box = Box::new(Orientation::Vertical, 6);
        view_controls_box.set_margin_start(6);
        view_controls_box.set_margin_end(6);
        view_controls_box.set_margin_top(6);
        view_controls_box.set_margin_bottom(6);

        // Grid toggle
        let grid_toggle = gtk4::CheckButton::with_label(&t!("Show Grid"));
        grid_toggle.set_active(true);
        {
            let state_grid = state.clone();
            let canvas_grid = canvas.clone();
            grid_toggle.connect_toggled(move |btn| {
                state_grid.borrow_mut().show_grid = btn.is_active();
                canvas_grid.widget.queue_draw();
            });
        }
        view_controls_box.append(&grid_toggle);

        // Grid spacing
        let system = settings_controller
            .persistence
            .borrow()
            .config()
            .ui
            .measurement_system;
        let unit_label = gcodekit5_core::units::get_unit_label(system);

        let grid_spacing_combo = gtk4::ComboBoxText::new();
        grid_spacing_combo.set_hexpand(true);
        grid_spacing_combo.set_tooltip_text(Some(&t!("Grid spacing")));

        for mm in [1.0_f64, 5.0, 10.0, 25.0, 50.0] {
            let label = format!(
                "{} {}",
                gcodekit5_core::units::format_length(mm as f32, system),
                unit_label
            );
            grid_spacing_combo.append(Some(&mm.to_string()), &label);
        }

        grid_spacing_combo.set_active_id(Some("10"));
        {
            let state_grid_spacing = state.clone();
            let canvas_grid_spacing = canvas.clone();
            grid_spacing_combo.connect_changed(move |cb| {
                if let Some(id) = cb.active_id() {
                    if let Ok(mm) = id.parse::<f64>() {
                        state_grid_spacing.borrow_mut().grid_spacing_mm = mm;
                        canvas_grid_spacing.widget.queue_draw();
                    }
                }
            });
        }

        let grid_spacing_row = Box::new(Orientation::Horizontal, 6);
        grid_spacing_row.append(&Label::new(Some(&t!("Grid spacing"))));
        grid_spacing_row.append(&grid_spacing_combo);
        view_controls_box.append(&grid_spacing_row);

        // Snap controls
        let snap_toggle = gtk4::CheckButton::with_label(&t!("Snap"));
        snap_toggle.set_tooltip_text(Some(&t!("Snap to grid")));
        snap_toggle.set_active(state.borrow().snap_enabled);
        {
            let state_snap = state.clone();
            snap_toggle.connect_toggled(move |btn| {
                state_snap.borrow_mut().snap_enabled = btn.is_active();
            });
        }
        view_controls_box.append(&snap_toggle);

        let snap_threshold = gtk4::SpinButton::with_range(0.0, 5.0, 0.1);
        snap_threshold.set_tooltip_text(Some(&t!("Snap threshold")));
        let snap_display = match system {
            gcodekit5_core::units::MeasurementSystem::Metric => state.borrow().snap_threshold_mm,
            gcodekit5_core::units::MeasurementSystem::Imperial => {
                state.borrow().snap_threshold_mm / 25.4
            }
        };
        snap_threshold.set_value(snap_display);
        {
            let state_snap = state.clone();
            snap_threshold.connect_value_changed(move |sp| {
                let val = sp.value();
                let mm = match system {
                    gcodekit5_core::units::MeasurementSystem::Metric => val,
                    gcodekit5_core::units::MeasurementSystem::Imperial => val * 25.4,
                };
                state_snap.borrow_mut().snap_threshold_mm = mm.max(0.0);
            });
        }

        let snap_threshold_row = Box::new(Orientation::Horizontal, 6);
        snap_threshold_row.append(&Label::new(Some(&t!("Snap threshold"))));
        snap_threshold_row.append(&snap_threshold);
        view_controls_box.append(&snap_threshold_row);

        // Toolpath toggle
        let toolpath_toggle = gtk4::CheckButton::with_label(&t!("Show Toolpaths"));
        toolpath_toggle.set_active(false);
        {
            let state_toolpath = state.clone();
            let canvas_toolpath = canvas.clone();
            toolpath_toggle.connect_toggled(move |btn| {
                let active = btn.is_active();
                state_toolpath.borrow_mut().show_toolpaths = active;
                if active {
                    canvas_toolpath.generate_preview_toolpaths();
                } else {
                    canvas_toolpath.widget.queue_draw();
                }
            });
        }
        view_controls_box.append(&toolpath_toggle);

        // Preview generation progress + cancel
        let preview_spinner = gtk4::Spinner::new();
        preview_spinner.set_visible(false);

        let preview_cancel_btn = gtk4::Button::builder()
            .icon_name("process-stop-symbolic")
            .tooltip_text(t!("Cancel"))
            .build();
        preview_cancel_btn.set_visible(false);
        preview_cancel_btn.update_property(&[gtk4::accessible::Property::Label(&t!("Cancel"))]);

        {
            let cancel_flag = canvas.preview_cancel.clone();
            let generating = canvas.preview_generating.clone();
            preview_cancel_btn.connect_clicked(move |_| {
                cancel_flag.store(true, Ordering::SeqCst);
                generating.set(false);
            });
        }

        let preview_row = Box::new(Orientation::Horizontal, 6);
        preview_row.append(&preview_spinner);
        preview_row.append(&preview_cancel_btn);
        view_controls_box.append(&preview_row);

        // Keep widgets in sync with generating state
        {
            let generating = canvas.preview_generating.clone();
            let spinner = preview_spinner;
            let cancel_btn = preview_cancel_btn;
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                let on = generating.get();
                spinner.set_visible(on);
                cancel_btn.set_visible(on);
                if on {
                    spinner.start();
                } else {
                    spinner.stop();
                }
                gtk4::glib::ControlFlow::Continue
            });
        }

        gtk4::Expander::builder()
            .label(t!("View Controls"))
            .expanded(true)
            .child(&view_controls_box)
            .build()
    }

    /// Sets up keyboard shortcuts for the designer canvas
    fn setup_keyboard_shortcuts(canvas: &Rc<DesignerCanvas>) {
        let key_controller = EventControllerKey::new();
        let canvas_keys = canvas.clone();
        key_controller.connect_key_pressed(move |_, key, _code, modifiers| {
            let ctrl = modifiers.contains(ModifierType::CONTROL_MASK);
            let alt = modifiers.contains(ModifierType::ALT_MASK);

            match (key, ctrl, alt) {
                // Ctrl+Z - Undo
                (Key::z, true, _) | (Key::Z, true, _) => {
                    canvas_keys.undo();
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+Y - Redo
                (Key::y, true, _) | (Key::Y, true, _) => {
                    canvas_keys.redo();
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+C - Copy
                (Key::c, true, _) | (Key::C, true, _) => {
                    canvas_keys.copy_selected();
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+V - Paste
                (Key::v, true, false) | (Key::V, true, false) => {
                    canvas_keys.paste();
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+D - Duplicate
                (Key::d, true, _) | (Key::D, true, _) => {
                    canvas_keys.duplicate_selected();
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+G - Group (Shift+G for Ungroup)
                (Key::g, true, _) | (Key::G, true, _) => {
                    if modifiers.contains(ModifierType::SHIFT_MASK) {
                        canvas_keys.ungroup_selected();
                    } else {
                        canvas_keys.group_selected();
                    }
                    gtk4::glib::Propagation::Stop
                }
                // Ctrl+U - Ungroup
                (Key::u, true, _) | (Key::U, true, _) => {
                    canvas_keys.ungroup_selected();
                    gtk4::glib::Propagation::Stop
                }
                // Delete or Backspace - Delete selected
                (Key::Delete, _, _) | (Key::BackSpace, _, _) => {
                    canvas_keys.delete_selected();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+L - Align Left
                (Key::l, false, true) | (Key::L, false, true) => {
                    canvas_keys.align_left();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+R - Align Right
                (Key::r, false, true) | (Key::R, false, true) => {
                    canvas_keys.align_right();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+T - Align Top
                (Key::t, false, true) | (Key::T, false, true) => {
                    canvas_keys.align_top();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+B - Align Bottom
                (Key::b, false, true) | (Key::B, false, true) => {
                    canvas_keys.align_bottom();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+H - Align Center Horizontal
                (Key::h, false, true) | (Key::H, false, true) => {
                    canvas_keys.align_center_horizontal();
                    gtk4::glib::Propagation::Stop
                }
                // Alt+V - Align Center Vertical
                (Key::v, false, true) | (Key::V, false, true) => {
                    canvas_keys.align_center_vertical();
                    gtk4::glib::Propagation::Stop
                }
                // View shortcuts (no Ctrl/Alt)
                (Key::plus, false, false)
                | (Key::KP_Add, false, false)
                | (Key::equal, false, false) => {
                    canvas_keys.zoom_in();
                    gtk4::glib::Propagation::Stop
                }
                (Key::minus, false, false)
                | (Key::KP_Subtract, false, false)
                | (Key::underscore, false, false) => {
                    canvas_keys.zoom_out();
                    gtk4::glib::Propagation::Stop
                }
                (Key::f, false, false) | (Key::F, false, false) => {
                    canvas_keys.zoom_fit();
                    gtk4::glib::Propagation::Stop
                }
                (Key::r, false, false) | (Key::R, false, false) => {
                    canvas_keys.reset_view();
                    gtk4::glib::Propagation::Stop
                }
                (Key::d, false, false) | (Key::D, false, false) => {
                    canvas_keys.fit_to_device_area();
                    canvas_keys.widget.queue_draw();
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });

        canvas.widget.set_focusable(true);
        canvas.widget.set_can_focus(true);
        canvas.widget.add_controller(key_controller);

        // Grab focus on canvas when clicked
        let canvas_focus = canvas.clone();
        let click_for_focus = GestureClick::new();
        click_for_focus.set_button(1);
        click_for_focus.connect_pressed(move |_, _, _, _| {
            canvas_focus.widget.grab_focus();
        });
        canvas.widget.add_controller(click_for_focus);
    }

    /// Creates the floating zoom/view controls for the canvas overlay.
    /// Returns (floating_box, zoom_in, zoom_out, fit, reset, fit_device, scrollbars_btn)
    #[allow(clippy::type_complexity)]
    fn create_floating_controls(
        has_device_manager: bool,
    ) -> (
        Box,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
    ) {
        let floating_box = Box::new(Orientation::Horizontal, 4);
        floating_box.add_css_class("visualizer-osd");
        floating_box.add_css_class("osd-controls");
        floating_box.set_halign(gtk4::Align::End);
        floating_box.set_valign(gtk4::Align::End);
        floating_box.set_margin_bottom(20);
        floating_box.set_margin_end(20);

        let float_zoom_out = gtk4::Button::builder()
            .icon_name("zoom-out-symbolic")
            .tooltip_text(t!("Zoom Out"))
            .build();
        float_zoom_out.update_property(&[gtk4::accessible::Property::Label(&t!("Zoom Out"))]);

        let float_fit = gtk4::Button::builder()
            .icon_name("zoom-fit-best-symbolic")
            .tooltip_text(t!("Fit to Content"))
            .build();
        float_fit.update_property(&[gtk4::accessible::Property::Label(&t!("Fit to Content"))]);

        let float_reset = gtk4::Button::builder()
            .icon_name("view-restore-symbolic")
            .tooltip_text(t!("Fit to Viewport"))
            .build();
        float_reset.update_property(&[gtk4::accessible::Property::Label(&t!("Fit to Viewport"))]);

        let float_fit_device = gtk4::Button::builder()
            .icon_name("preferences-desktop-display-symbolic")
            .tooltip_text(t!("Fit to Device Working Area"))
            .build();
        float_fit_device.update_property(&[gtk4::accessible::Property::Label(&t!(
            "Fit to Device Working Area"
        ))]);

        let scrollbars_btn = gtk4::Button::builder()
            .icon_name("view-list-symbolic")
            .tooltip_text(t!("Toggle Scrollbars"))
            .build();
        scrollbars_btn
            .update_property(&[gtk4::accessible::Property::Label(&t!("Toggle Scrollbars"))]);

        let help_btn = gtk4::Button::builder()
            .label("?")
            .tooltip_text(t!("Keyboard Shortcuts"))
            .build();
        help_btn.update_property(&[gtk4::accessible::Property::Label(&t!("Keyboard Shortcuts"))]);

        let help_popover = Popover::new();
        help_popover.set_parent(&help_btn);
        help_popover.set_has_arrow(true);
        {
            let help_box = Box::new(Orientation::Vertical, 6);
            help_box.set_margin_start(12);
            help_box.set_margin_end(12);
            help_box.set_margin_top(12);
            help_box.set_margin_bottom(12);
            help_box.append(&Label::new(Some(&t!("Designer shortcuts"))));
            help_box.append(&Label::new(Some("Ctrl+Z / Ctrl+Y  —  Undo / Redo")));
            help_box.append(&Label::new(Some("Ctrl+C / Ctrl+V  —  Copy / Paste")));
            help_box.append(&Label::new(Some("Delete  —  Delete selection")));
            help_box.append(&Label::new(Some("+ / -  —  Zoom")));
            help_box.append(&Label::new(Some("F  —  Fit to Content")));
            help_box.append(&Label::new(Some("R  —  Fit to Viewport")));
            help_box.append(&Label::new(Some("D  —  Fit to Device Working Area")));
            help_box.append(&Label::new(Some(&t!("Right click for context menu"))));
            help_popover.set_child(Some(&help_box));
        }
        {
            let pop = help_popover.clone();
            help_btn.connect_clicked(move |_| pop.popup());
        }

        let float_zoom_in = gtk4::Button::builder()
            .icon_name("zoom-in-symbolic")
            .tooltip_text(t!("Zoom In"))
            .build();
        float_zoom_in.update_property(&[gtk4::accessible::Property::Label(&t!("Zoom In"))]);

        for b in [
            &float_zoom_out,
            &float_fit,
            &float_reset,
            &float_fit_device,
            &scrollbars_btn,
            &help_btn,
            &float_zoom_in,
        ] {
            b.set_size_request(28, 28);
        }

        floating_box.append(&float_zoom_out);
        floating_box.append(&float_fit);
        floating_box.append(&float_reset);
        if has_device_manager {
            floating_box.append(&float_fit_device);
        }
        floating_box.append(&scrollbars_btn);
        floating_box.append(&help_btn);
        floating_box.append(&float_zoom_in);

        (
            floating_box,
            float_zoom_in,
            float_zoom_out,
            float_fit,
            float_reset,
            float_fit_device,
            scrollbars_btn,
        )
    }

    /// Creates the empty state overlay shown when no shapes exist.
    /// Returns (empty_box, new_btn, open_btn, svg_btn, dxf_btn, stl_btn)
    #[allow(clippy::type_complexity)]
    fn create_empty_state(
        settings_controller: &Rc<SettingsController>,
    ) -> (
        Box,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
        gtk4::Button,
    ) {
        let empty_box = Box::new(Orientation::Vertical, 8);
        empty_box.add_css_class("visualizer-osd");
        empty_box.set_halign(gtk4::Align::Center);
        empty_box.set_valign(gtk4::Align::Center);
        empty_box.set_margin_start(20);
        empty_box.set_margin_end(20);
        empty_box.set_margin_top(20);
        empty_box.set_margin_bottom(20);
        empty_box.append(&Label::new(Some(&t!("No shapes yet"))));
        empty_box.append(&Label::new(Some(&t!(
            "Use the toolbox on the left to start drawing."
        ))));

        let empty_actions = Box::new(Orientation::Horizontal, 8);
        empty_actions.set_halign(gtk4::Align::Center);

        let empty_new_btn = gtk4::Button::with_label(&t!("New"));
        empty_new_btn.add_css_class("suggested-action");
        let empty_open_btn = gtk4::Button::with_label(&t!("Load Design"));
        let empty_import_svg_btn = gtk4::Button::with_label(&t!("Import SVG"));
        let empty_import_dxf_btn = gtk4::Button::with_label(&t!("Import DXF"));
        let empty_import_stl_btn = gtk4::Button::with_label(&t!("Import STL"));

        let enable_stl_import = settings_controller
            .persistence
            .borrow()
            .config()
            .ui
            .enable_stl_import;
        empty_import_stl_btn.set_visible(enable_stl_import);

        empty_actions.append(&empty_new_btn);
        empty_actions.append(&empty_open_btn);
        empty_actions.append(&empty_import_svg_btn);
        empty_actions.append(&empty_import_dxf_btn);
        empty_actions.append(&empty_import_stl_btn);
        empty_box.append(&empty_actions);
        empty_box.set_visible(true);

        (
            empty_box,
            empty_new_btn,
            empty_open_btn,
            empty_import_svg_btn,
            empty_import_dxf_btn,
            empty_import_stl_btn,
        )
    }

    /// Creates the status panel for the bottom-left of the canvas.
    /// Returns (status_box, status_label, units_badge)
    fn create_status_panel() -> (Box, Label, Label) {
        let status_box = Box::new(Orientation::Horizontal, 4);
        status_box.add_css_class("visualizer-osd");
        status_box.set_halign(gtk4::Align::Start);
        status_box.set_valign(gtk4::Align::End);
        status_box.set_margin_bottom(20);
        status_box.set_margin_start(20);

        let status_label_osd = Label::builder().label(" ").build();
        status_label_osd.set_hexpand(true);

        let units_badge = Label::new(Some(""));
        units_badge.add_css_class("osd-units-badge");

        status_box.append(&status_label_osd);
        status_box.append(&units_badge);

        (status_box, status_label_osd, units_badge)
    }

    /// Starts the status OSD update loop
    fn start_status_update_loop(
        status_label: Label,
        units_badge: Label,
        empty_box: Box,
        canvas: Rc<DesignerCanvas>,
        settings_controller: Rc<SettingsController>,
    ) {
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            let state = canvas.state.borrow();
            let zoom = state.canvas.zoom();
            let pan_x = state.canvas.pan_x();
            let pan_y = state.canvas.pan_y();
            let has_shapes = !state.canvas.shape_store.is_empty();
            let snap_on = state.snap_enabled;
            drop(state);

            let constraint_on = *canvas.shift_pressed.borrow();

            let width = canvas.widget.width() as f64;
            let height = canvas.widget.height() as f64;

            let center_x = ((width / 2.0) - pan_x) / zoom;
            let center_y = ((height / 2.0) - pan_y) / zoom;

            let (cursor_x, cursor_y) = *canvas.mouse_pos.borrow();

            let system = settings_controller
                .persistence
                .borrow()
                .config()
                .ui
                .measurement_system;
            let mut status = format_zoom_center_cursor(
                zoom,
                center_x as f32,
                center_y as f32,
                cursor_x as f32,
                cursor_y as f32,
                system,
            );

            if snap_on || constraint_on {
                let mut parts: Vec<String> = Vec::new();
                if snap_on {
                    parts.push(t!("Snap"));
                }
                if constraint_on {
                    parts.push(t!("Constraint"));
                }
                status.push_str(&format!("  {}", parts.join(" / ")));
            }

            status_label.set_text(&status);
            units_badge.set_text(gcodekit5_core::units::get_unit_label(system));
            empty_box.set_visible(!has_shapes);

            gtk4::glib::ControlFlow::Continue
        });
    }

    pub fn current_tool(&self) -> DesignerTool {
        self.toolbox.current_tool()
    }

    pub fn set_tool(&self, tool: DesignerTool) {
        self.toolbox.set_tool(tool);
    }

    pub fn set_status(&self, message: &str) {
        self.status_label.set_text(message);
    }

    pub fn set_on_gcode_generated<F: Fn(String) + 'static>(&self, f: F) {
        *self.on_gcode_generated.borrow_mut() = Some(std::boxed::Box::new(f));
    }

    pub fn fit_to_device(&self) {
        self.canvas.fit_to_device_area();
        self.canvas.widget.queue_draw();
    }

    pub fn undo(&self) {
        self.canvas.undo();
    }

    pub fn redo(&self) {
        self.canvas.redo();
    }

    pub fn cut(&self) {
        self.canvas.copy_selected();
        self.canvas.delete_selected();
    }

    pub fn copy(&self) {
        self.canvas.copy_selected();
    }

    pub fn paste(&self) {
        self.canvas.paste();
    }

    pub fn delete(&self) {
        self.canvas.delete_selected();
    }

    /// Queue a redraw of the designer canvas
    pub fn queue_draw(&self) {
        self.canvas.widget.queue_draw();
    }

    pub fn new_file(&self) {
        let mut state = self.canvas.state.borrow_mut();
        state.canvas.clear();
        *self.current_file.borrow_mut() = None;
        drop(state);

        // Refresh layers
        self.layers.refresh(&self.canvas.state);
        self.canvas.widget.queue_draw();
        self.set_status(&t!("New design created"));
    }

    pub fn open_file(&self) {
        let dialog = FileChooserNative::builder()
            .title(t!("Open Design File"))
            .action(FileChooserAction::Open)
            .modal(true)
            .build();

        if let Some(root) = self.widget.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }

        // Set initial directory from settings
        if let Some(ref settings) = self.settings_persistence {
            if let Ok(settings_ref) = settings.try_borrow() {
                let default_dir = &settings_ref.config().file_processing.output_directory;
                if default_dir.exists() {
                    let file = gtk4::gio::File::for_path(default_dir);
                    let _ = dialog.set_current_folder(Some(&file));
                }
            }
        }

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&t!("GCodeKit Design Files")));
        filter.add_pattern("*.gckd");
        filter.add_pattern("*.gck5");
        dialog.add_filter(&filter);

        let all_filter = gtk4::FileFilter::new();
        all_filter.set_name(Some(&t!("All Files")));
        all_filter.add_pattern("*");
        dialog.add_filter(&all_filter);

        let canvas = self.canvas.clone();
        let current_file = self.current_file.clone();
        let layers = self.layers.clone();
        let status_label = self.status_label.clone();
        let toolbox = self.toolbox.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        match DesignFile::load_from_file(&path) {
                            Ok(design) => {
                                let mut state = canvas.state.borrow_mut();
                                state.canvas.clear();

                                let mut max_id = 0;
                                let mut restored_shapes: usize = 0;
                                for shape_data in design.shapes {
                                    let id = shape_data.id as u64;
                                    if id > max_id {
                                        max_id = id;
                                    }

                                    if let Ok(obj) =
                                        DesignFile::to_drawing_object(&shape_data, id as i32)
                                    {
                                        state.canvas.restore_shape(obj);
                                        restored_shapes += 1;
                                    }
                                }

                                state.canvas.set_next_id(max_id + 1);

                                // Restore tool settings from design file
                                state.tool_settings.feed_rate = design.toolpath_params.feed_rate;
                                state.tool_settings.spindle_speed =
                                    design.toolpath_params.spindle_speed as u32;
                                state.tool_settings.tool_diameter =
                                    design.toolpath_params.tool_diameter;
                                state.tool_settings.cut_depth = design.toolpath_params.cut_depth;

                                // Also update the toolpath generator to match
                                state
                                    .toolpath_generator
                                    .set_feed_rate(design.toolpath_params.feed_rate);
                                state
                                    .toolpath_generator
                                    .set_spindle_speed(design.toolpath_params.spindle_speed as u32);
                                state
                                    .toolpath_generator
                                    .set_tool_diameter(design.toolpath_params.tool_diameter);
                                state
                                    .toolpath_generator
                                    .set_cut_depth(design.toolpath_params.cut_depth);

                                // Restore stock parameters from design file (create if needed)
                                state.stock_material = Some(StockMaterial {
                                    width: design.toolpath_params.stock_width,
                                    height: design.toolpath_params.stock_height,
                                    thickness: design.toolpath_params.stock_thickness,
                                    origin: (0.0, 0.0, 0.0),
                                    safe_z: design.toolpath_params.safe_z_height,
                                });

                                // Update viewport (fallback to fit if invalid)
                                let zoom = design.viewport.zoom;
                                let pan_x = design.viewport.pan_x;
                                let pan_y = design.viewport.pan_y;
                                let viewport_ok = zoom.is_finite()
                                    && zoom > 0.0001
                                    && pan_x.is_finite()
                                    && pan_y.is_finite();
                                if viewport_ok {
                                    state.canvas.set_zoom(zoom);
                                    state.canvas.set_pan(pan_x, pan_y);
                                }

                                *current_file.borrow_mut() = Some(path.clone());
                                drop(state);

                                // If the saved viewport is missing/degenerate, frame the loaded geometry.
                                if restored_shapes > 0 && !viewport_ok {
                                    canvas.zoom_fit();
                                }

                                layers.refresh(&canvas.state);
                                // Refresh tool/stock settings UI to show loaded values
                                toolbox.refresh_settings();
                                canvas.widget.queue_draw();
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Loaded:"),
                                    path.display()
                                ));
                            }
                            Err(e) => {
                                error!("Error loading file: {}", e);
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Error loading file:"),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    fn import_file_internal(&self, kind: Option<&'static str>) {
        let title = match kind {
            Some("svg") => t!("Import SVG File"),
            Some("dxf") => t!("Import DXF File"),
            Some("stl") => t!("Import STL File (3D Shadow)"),
            _ => t!("Import Design File"),
        };

        let dialog = FileChooserNative::builder()
            .title(title)
            .action(FileChooserAction::Open)
            .modal(true)
            .build();

        if let Some(root) = self.widget.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }

        // Set initial directory from settings
        if let Some(ref settings) = self.settings_persistence {
            if let Ok(settings_ref) = settings.try_borrow() {
                let default_dir = &settings_ref.config().file_processing.output_directory;
                if default_dir.exists() {
                    let file = gtk4::gio::File::for_path(default_dir);
                    let _ = dialog.set_current_folder(Some(&file));
                }
            }
        }

        // Check STL import setting for STL support
        let enable_stl_import = if let Some(ref settings) = self.settings_persistence {
            if let Ok(settings_ref) = settings.try_borrow() {
                settings_ref.config().ui.enable_stl_import
            } else {
                false
            }
        } else {
            false
        };

        match kind {
            Some("svg") => {
                let svg_filter = gtk4::FileFilter::new();
                svg_filter.set_name(Some(&t!("SVG Files")));
                svg_filter.add_pattern("*.svg");
                dialog.add_filter(&svg_filter);
            }
            Some("dxf") => {
                let dxf_filter = gtk4::FileFilter::new();
                dxf_filter.set_name(Some(&t!("DXF Files")));
                dxf_filter.add_pattern("*.dxf");
                dialog.add_filter(&dxf_filter);
            }
            Some("stl") => {
                // Only show STL filter if STL import is enabled
                if enable_stl_import {
                    let stl_filter = gtk4::FileFilter::new();
                    stl_filter.set_name(Some(&t!("STL Files")));
                    stl_filter.add_pattern("*.stl");
                    dialog.add_filter(&stl_filter);
                }
            }
            _ => {
                let filter = gtk4::FileFilter::new();
                filter.set_name(Some(&t!("Supported Files")));
                filter.add_pattern("*.svg");
                filter.add_pattern("*.dxf");
                if enable_stl_import {
                    filter.add_pattern("*.stl");
                }
                dialog.add_filter(&filter);

                let svg_filter = gtk4::FileFilter::new();
                svg_filter.set_name(Some(&t!("SVG Files")));
                svg_filter.add_pattern("*.svg");
                dialog.add_filter(&svg_filter);

                let dxf_filter = gtk4::FileFilter::new();
                dxf_filter.set_name(Some(&t!("DXF Files")));
                dxf_filter.add_pattern("*.dxf");
                dialog.add_filter(&dxf_filter);

                if enable_stl_import {
                    let stl_filter = gtk4::FileFilter::new();
                    stl_filter.set_name(Some(&t!("STL Files")));
                    stl_filter.add_pattern("*.stl");
                    dialog.add_filter(&stl_filter);
                }
            }
        }

        let all_filter = gtk4::FileFilter::new();
        all_filter.set_name(Some(&t!("All Files")));
        all_filter.add_pattern("*");
        dialog.add_filter(&all_filter);

        let canvas = self.canvas.clone();
        let layers = self.layers.clone();
        let status_label = self.status_label.clone();
        let settings_persistence = self.settings_persistence.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        // Check STL import setting for STL processing
                        let enable_stl_import = if let Some(ref settings) = settings_persistence {
                            if let Ok(settings_ref) = settings.try_borrow() {
                                settings_ref.config().ui.enable_stl_import
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        let result = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            match ext.to_lowercase().as_str() {
                                "svg" => match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let importer = gcodekit5_designer::import::SvgImporter::new(
                                            1.0, 0.0, 0.0,
                                        );
                                        importer.import_string(&content)
                                    }
                                    Err(e) => Err(anyhow::anyhow!("Failed to read file: {}", e)),
                                },
                                "dxf" => {
                                    let importer =
                                        gcodekit5_designer::import::DxfImporter::new(1.0, 0.0, 0.0);
                                    importer.import_file(path.to_str().unwrap_or(""))
                                }
                                "stl" => {
                                    // Only allow STL import if STL import is enabled
                                    if enable_stl_import {
                                        let importer =
                                            gcodekit5_designer::import::StlImporter::new()
                                                .with_scale(1.0)
                                                .with_centering(true);

                                        // Import STL and create shadow projection
                                        let result = importer.import_file(path.to_str().unwrap_or(""));

                                        // TODO(#16): Add 3D mesh to visualizer for preview
                                        // This would integrate with the new Scene3D system:
                                        // if let Ok(ref design) = result {
                                        //     if let Some(mesh_3d) = &design.mesh_3d {
                                        //         // Add to 3D scene for preview
                                        //         // Show 3D visualization panel
                                        //     }
                                        // }

                                        result
                                    } else {
                                        Err(anyhow::anyhow!("STL import requires the STL import feature to be enabled in settings"))
                                    }
                                }
                                _ => Err(anyhow::anyhow!("Unsupported file format")),
                            }
                        } else {
                            Err(anyhow::anyhow!("Unknown file extension"))
                        };

                        match result {
                            Ok(design) => {
                                let mut state = canvas.state.borrow_mut();

                                // Add imported shapes to canvas
                                for shape in design.shapes {
                                    state.add_shape_with_undo(shape);
                                }

                                drop(state);

                                // Make imported geometry visible immediately
                                canvas.zoom_fit();

                                layers.refresh(&canvas.state);
                                canvas.widget.queue_draw();
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Imported:"),
                                    path.display()
                                ));
                            }
                            Err(e) => {
                                error!("Error importing file: {}", e);
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Error importing file:"),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    pub fn import_file(&self) {
        self.import_file_internal(None);
    }

    pub fn import_svg_file(&self) {
        self.import_file_internal(Some("svg"));
    }

    pub fn import_dxf_file(&self) {
        self.import_file_internal(Some("dxf"));
    }

    pub fn import_stl_file(&self) {
        self.import_file_internal(Some("stl"));
    }

    pub fn save_file(&self) {
        let current_path = self.current_file.borrow().clone();

        if let Some(path) = current_path {
            self.save_to_path(path);
        } else {
            self.save_as_file();
        }
    }

    pub fn save_as_file(&self) {
        let dialog = FileChooserNative::builder()
            .title("Save Design File")
            .action(FileChooserAction::Save)
            .modal(true)
            .build();

        if let Some(root) = self.widget.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }

        // Set initial directory from settings
        if let Some(ref settings) = self.settings_persistence {
            if let Ok(settings_ref) = settings.try_borrow() {
                let default_dir = &settings_ref.config().file_processing.output_directory;
                if default_dir.exists() {
                    let file = gtk4::gio::File::for_path(default_dir);
                    let _ = dialog.set_current_folder(Some(&file));
                }
            }
        }

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&t!("GCodeKit Design Files")));
        filter.add_pattern("*.gckd");
        dialog.add_filter(&filter);

        let canvas = self.canvas.clone();
        let current_file = self.current_file.clone();
        let status_label = self.status_label.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(mut path) = file.path() {
                        if path.extension().is_none() {
                            path.set_extension("gckd");
                        }

                        // Save logic
                        let state = canvas.state.borrow();
                        let mut design =
                            DesignFile::new(path.file_stem().unwrap_or_default().to_string_lossy());

                        // Viewport
                        design.viewport.zoom = state.canvas.zoom();
                        design.viewport.pan_x = state.canvas.pan_x();
                        design.viewport.pan_y = state.canvas.pan_y();

                        // Tool settings
                        design.toolpath_params.feed_rate = state.tool_settings.feed_rate;
                        design.toolpath_params.spindle_speed =
                            state.tool_settings.spindle_speed as f64;
                        design.toolpath_params.tool_diameter = state.tool_settings.tool_diameter;
                        design.toolpath_params.cut_depth = state.tool_settings.cut_depth;

                        // Stock and toolpath parameters
                        if let Some(ref stock) = state.stock_material {
                            design.toolpath_params.stock_width = stock.width;
                            design.toolpath_params.stock_height = stock.height;
                            design.toolpath_params.stock_thickness = stock.thickness;
                            design.toolpath_params.safe_z_height = stock.safe_z;
                        }

                        // Shapes
                        for obj in state.canvas.shapes() {
                            let shape_data = DesignFile::from_drawing_object(obj);
                            design.shapes.push(shape_data);
                        }

                        match design.save_to_file(&path) {
                            Ok(_) => {
                                *current_file.borrow_mut() = Some(path.clone());
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Saved:"),
                                    path.display()
                                ));
                            }
                            Err(e) => {
                                error!("Error saving file: {}", e);
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Error saving file:"),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    fn save_to_path(&self, path: PathBuf) {
        let state = self.canvas.state.borrow();
        let mut design = DesignFile::new(path.file_stem().unwrap_or_default().to_string_lossy());

        // Viewport
        design.viewport.zoom = state.canvas.zoom();
        design.viewport.pan_x = state.canvas.pan_x();
        design.viewport.pan_y = state.canvas.pan_y();

        // Tool settings
        design.toolpath_params.feed_rate = state.tool_settings.feed_rate;
        design.toolpath_params.spindle_speed = state.tool_settings.spindle_speed as f64;
        design.toolpath_params.tool_diameter = state.tool_settings.tool_diameter;
        design.toolpath_params.cut_depth = state.tool_settings.cut_depth;

        // Stock and toolpath parameters
        if let Some(ref stock) = state.stock_material {
            design.toolpath_params.stock_width = stock.width;
            design.toolpath_params.stock_height = stock.height;
            design.toolpath_params.stock_thickness = stock.thickness;
            design.toolpath_params.safe_z_height = stock.safe_z;
        }

        // Shapes
        for obj in state.canvas.shapes() {
            let shape_data = DesignFile::from_drawing_object(obj);
            design.shapes.push(shape_data);
        }

        match design.save_to_file(&path) {
            Ok(_) => {
                self.set_status(&format!("{} {}", t!("Saved:"), path.display()));
            }
            Err(e) => {
                error!("Error saving file: {}", e);
                self.set_status(&format!("{} {}", t!("Error saving file:"), e));
            }
        }
    }

    pub fn export_gcode(&self) {
        let window = self
            .widget
            .root()
            .and_then(|w| w.downcast::<gtk4::Window>().ok());
        let dialog = FileChooserNative::new(
            Some("Export G-Code"),
            window.as_ref(),
            FileChooserAction::Save,
            Some("Export"),
            Some("Cancel"),
        );

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("G-Code Files"));
        filter.add_pattern("*.nc");
        filter.add_pattern("*.gcode");
        filter.add_pattern("*.gc");
        dialog.add_filter(&filter);

        let canvas = self.canvas.clone();
        let status_label = self.status_label.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(mut path) = file.path() {
                        if path.extension().is_none() {
                            path.set_extension("nc");
                        }

                        // Generate G-code
                        let mut state = canvas.state.borrow_mut();

                        // Copy settings to avoid borrow issues
                        let feed_rate = state.tool_settings.feed_rate;
                        let spindle_speed = state.tool_settings.spindle_speed;
                        let tool_diameter = state.tool_settings.tool_diameter;
                        let cut_depth = state.tool_settings.cut_depth;
                        let start_depth = state.tool_settings.start_depth;

                        // Update toolpath generator settings from state
                        state.toolpath_generator.set_feed_rate(feed_rate);
                        state.toolpath_generator.set_spindle_speed(spindle_speed);
                        state.toolpath_generator.set_tool_diameter(tool_diameter);
                        state.toolpath_generator.set_cut_depth(cut_depth);
                        state.toolpath_generator.set_start_depth(start_depth);
                        state.toolpath_generator.set_step_in(tool_diameter * 0.4); // Default stepover

                        let gcode = state.generate_gcode();

                        match std::fs::write(&path, gcode) {
                            Ok(_) => {
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Exported G-Code:"),
                                    path.display()
                                ));
                            }
                            Err(e) => {
                                error!("Error exporting G-Code: {}", e);
                                status_label.set_text(&format!(
                                    "{} {}",
                                    t!("Error exporting G-Code:"),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    pub fn export_svg(&self) {
        let window = self
            .widget
            .root()
            .and_then(|w| w.downcast::<gtk4::Window>().ok());
        let dialog = FileChooserNative::new(
            Some("Export SVG"),
            window.as_ref(),
            FileChooserAction::Save,
            Some("Export"),
            Some("Cancel"),
        );

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("SVG Files"));
        filter.add_pattern("*.svg");
        dialog.add_filter(&filter);

        let canvas = self.canvas.clone();
        let status_label = self.status_label.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(mut path) = file.path() {
                        if path.extension().is_none() {
                            path.set_extension("svg");
                        }

                        let state = canvas.state.borrow();

                        // Calculate bounds
                        let mut min_x = f64::INFINITY;
                        let mut min_y = f64::INFINITY;
                        let mut max_x = f64::NEG_INFINITY;
                        let mut max_y = f64::NEG_INFINITY;

                        let shapes: Vec<_> = state.canvas.shapes().collect();
                        if shapes.is_empty() {
                            status_label.set_text(&t!("Nothing to export"));
                            dialog.destroy();
                            return;
                        }

                        for obj in &shapes {
                            let (x1, y1, x2, y2) = obj.get_effective_shape().bounds();
                            min_x = min_x.min(x1);
                            min_y = min_y.min(y1);
                            max_x = max_x.max(x2);
                            max_y = max_y.max(y2);
                        }

                        // Add some padding
                        let padding = 10.0;
                        min_x -= padding;
                        min_y -= padding;
                        max_x += padding;
                        max_y += padding;

                        let width = max_x - min_x;
                        let height = max_y - min_y;

                        let mut svg = String::new();
                        svg.push_str(&format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="{:.2}mm" height="{:.2}mm" viewBox="{:.2} {:.2} {:.2} {:.2}" xmlns="http://www.w3.org/2000/svg">
"#, width, height, min_x, min_y, width, height));

                        for obj in &shapes {
                            let style = "fill:none;stroke:black;stroke-width:0.5";
                            let effective_shape = obj.get_effective_shape();
                            match &effective_shape {
                                Shape::Rectangle(r) => {
                                    let x = r.center.x - r.width / 2.0;
                                    let y = r.center.y - r.height / 2.0;
                                    let effective_radius = r.effective_corner_radius();
                                    svg.push_str(&format!(r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="{:.2}" style="{}" transform="rotate({:.2} {:.2} {:.2})" />"#,
                                        x, y, r.width, r.height, effective_radius, style,
                                        r.rotation, r.center.x, r.center.y
                                    ));
                                }
                                Shape::Circle(c) => {
                                    svg.push_str(&format!(r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" style="{}" />"#,
                                        c.center.x, c.center.y, c.radius, style
                                    ));
                                }
                                Shape::Line(l) => {
                                    svg.push_str(&format!(r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" style="{}" transform="rotate({:.2} {:.2} {:.2})" />"#,
                                        l.start.x, l.start.y, l.end.x, l.end.y, style,
                                        l.rotation, (l.start.x+l.end.x)/2.0, (l.start.y+l.end.y)/2.0
                                    ));
                                }
                                Shape::Triangle(t) => {
                                    let path = t.render();
                                    let d = gcodekit5_designer::model::DesignPath::from_lyon_path(&path).to_svg_path();
                                    svg.push_str(&format!(r#"<path d="{}" style="{}" />"#, d, style));
                                }
                                Shape::Polygon(p) => {
                                    let path = p.render();
                                    let d = gcodekit5_designer::model::DesignPath::from_lyon_path(&path).to_svg_path();
                                    svg.push_str(&format!(r#"<path d="{}" style="{}" />"#, d, style));
                                }
                                Shape::Ellipse(e) => {
                                    svg.push_str(&format!(r#"<ellipse cx="{:.2}" cy="{:.2}" rx="{:.2}" ry="{:.2}" style="{}" transform="rotate({:.2} {:.2} {:.2})" />"#,
                                        e.center.x, e.center.y, e.rx, e.ry, style,
                                        e.rotation, e.center.x, e.center.y
                                    ));
                                }
                                Shape::Path(p) => {
                                    let mut d = String::new();
                                    let path = p.render();
                                    for event in path.iter() {
                                        match event {
                                            lyon::path::Event::Begin { at } => d.push_str(&format!("M {:.2} {:.2} ", at.x, at.y)),
                                            lyon::path::Event::Line { from: _, to } => d.push_str(&format!("L {:.2} {:.2} ", to.x, to.y)),
                                            lyon::path::Event::Quadratic { from: _, ctrl, to } => d.push_str(&format!("Q {:.2} {:.2} {:.2} {:.2} ", ctrl.x, ctrl.y, to.x, to.y)),
                                            lyon::path::Event::Cubic { from: _, ctrl1, ctrl2, to } => d.push_str(&format!("C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ", ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y)),
                                            lyon::path::Event::End { last: _, first: _, close } => if close { d.push_str("Z "); },
                                        }
                                    }
                                    let rect = lyon::algorithms::aabb::bounding_box(&path);
                                    let cx = (rect.min.x + rect.max.x) / 2.0;
                                    let cy = (rect.min.y + rect.max.y) / 2.0;

                                    svg.push_str(&format!(r#"<path d="{}" style="{}" transform="rotate({:.2} {:.2} {:.2})" />"#,
                                        d, style, p.rotation, cx, cy
                                    ));
                                }
                                Shape::Text(t) => {
                                    svg.push_str(&format!(r#"<text x="{:.2}" y="{:.2}" font-size="{:.2}" style="fill:black;stroke:none" transform="rotate({:.2} {:.2} {:.2})">{}</text>"#,
                                        t.x, t.y, t.font_size,
                                        t.rotation, t.x, t.y,
                                        t.text
                                    ));
                                }
                                Shape::Gear(g) => {
                                    let path = g.render();
                                    let d = gcodekit5_designer::model::DesignPath::from_lyon_path(&path).to_svg_path();
                                    svg.push_str(&format!(r#"<path d="{}" style="{}" />"#, d, style));
                                }
                                Shape::Sprocket(s) => {
                                    let path = s.render();
                                    let d = gcodekit5_designer::model::DesignPath::from_lyon_path(&path).to_svg_path();
                                    svg.push_str(&format!(r#"<path d="{}" style="{}" />"#, d, style));
                                }
                            }
                            svg.push('\n');
                        }

                        svg.push_str("</svg>");

                        match std::fs::write(&path, svg) {
                            Ok(_) => {
                                status_label.set_text(&format!("{} {}", t!("Exported SVG:"), path.display()));
                            }
                            Err(e) => {
                                error!("Error exporting SVG: {}", e);
                                status_label.set_text(&format!("{} {}", t!("Error exporting SVG:"), e));
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    // TODO(#17): File operations - Implement once shape structures are aligned
    // Phase 8 infrastructure is in place but needs shape struct updates

    pub fn add_shape(&self, shape: gcodekit5_designer::model::Shape) {
        let mut state = self.canvas.state.borrow_mut();
        state.add_shape_with_undo(shape);
        drop(state);
        self.layers.refresh(&self.canvas.state);
        self.canvas.widget.queue_draw();
    }
}
