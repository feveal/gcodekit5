use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, ComboBoxText, Entry, Label, Orientation, ScrolledWindow, 
    Stack, IconTheme, Image, GestureClick, Frame, Grid, FileChooserDialog, FileChooserAction,
    ResponseType,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::fs;

use gcodekit5_camtools::tabbed_box::{
    BoxParameters, FingerJointSettings, FingerStyle, BoxType, KeyDividerType, 
    TabbedBoxMaker as Generator
};

pub struct CamToolsView {
    pub content: Stack,
}

impl CamToolsView {
    pub fn new<F: Fn(String) + 'static>(on_generate: F) -> Self {
        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

        // Dashboard Page
        let dashboard = Self::create_dashboard(&stack);
        stack.add_named(&dashboard, Some("dashboard"));

        // Tool Pages
        let tabbed_box = TabbedBoxMaker::new(&stack, on_generate);
        stack.add_named(tabbed_box.widget(), Some("tabbed_box"));

        // Placeholders for other tools
        stack.add_named(&Self::create_placeholder("Jigsaw Puzzle Generator", &stack), Some("jigsaw"));
        stack.add_named(&Self::create_placeholder("Laser Image Engraver", &stack), Some("laser_image"));
        stack.add_named(&Self::create_placeholder("Laser Vector Engraver", &stack), Some("laser_vector"));
        stack.add_named(&Self::create_placeholder("Speeds & Feeds Calculator", &stack), Some("feeds"));
        stack.add_named(&Self::create_placeholder("Spoilboard Surfacing", &stack), Some("surfacing"));
        stack.add_named(&Self::create_placeholder("Spoilboard Grid", &stack), Some("grid"));

        Self { content: stack }
    }

    pub fn widget(&self) -> &Stack {
        &self.content
    }

    fn create_dashboard(stack: &Stack) -> ScrolledWindow {
        let container = Box::new(Orientation::Vertical, 24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.set_margin_start(24);
        container.set_margin_end(24);

        let title = Label::builder()
            .label("CAM Tools")
            .css_classes(vec!["title-1"])
            .halign(Align::Start)
            .build();
        container.append(&title);

        let grid = Grid::builder()
            .column_spacing(24)
            .row_spacing(24)
            .hexpand(true)
            .build();

        // Row 1
        grid.attach(&Self::create_tool_card(
            "Tabbed Box Maker",
            "Generate G-code for laser/CNC cut boxes with finger joints",
            "object-select-symbolic", // Placeholder icon
            "tabbed_box",
            stack
        ), 0, 0, 1, 1);

        grid.attach(&Self::create_tool_card(
            "Jigsaw Puzzle Generator",
            "Create custom jigsaw puzzle patterns from images",
            "image-x-generic-symbolic",
            "jigsaw",
            stack
        ), 1, 0, 1, 1);

        grid.attach(&Self::create_tool_card(
            "Laser Image Engraver",
            "Convert raster images to G-code for laser engraving",
            "camera-photo-symbolic",
            "laser_image",
            stack
        ), 2, 0, 1, 1);

        // Row 2
        grid.attach(&Self::create_tool_card(
            "Laser Vector Engraver",
            "Convert SVG and DXF vector files to G-code",
            "draw-bezier-curves-symbolic",
            "laser_vector",
            stack
        ), 0, 1, 1, 1);

        grid.attach(&Self::create_tool_card(
            "Speeds & Feeds Calculator",
            "Calculate optimal cutting speeds and feeds for your materials",
            "accessories-calculator-symbolic",
            "feeds",
            stack
        ), 1, 1, 1, 1);

        grid.attach(&Self::create_tool_card(
            "Spoilboard Surfacing",
            "Generate surfacing toolpaths to flatten your spoilboard",
            "view-refresh-symbolic",
            "surfacing",
            stack
        ), 2, 1, 1, 1);

        // Row 3
        grid.attach(&Self::create_tool_card(
            "Create Spoilboard Grid",
            "Generate grid patterns for spoilboard alignment",
            "view-grid-symbolic",
            "grid",
            stack
        ), 0, 2, 1, 1);

        container.append(&grid);

        ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&container)
            .build()
    }

    fn create_tool_card(title: &str, desc: &str, icon: &str, target_page: &str, stack: &Stack) -> Button {
        let button = Button::builder()
            .css_classes(vec!["card"])
            .hexpand(true)
            .vexpand(false)
            .build();

        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_width_request(250);
        content.set_height_request(200);

        let icon_img = Image::from_icon_name(icon);
        icon_img.set_pixel_size(64);
        icon_img.add_css_class("accent");
        
        let title_lbl = Label::builder()
            .label(title)
            .css_classes(vec!["heading"])
            .wrap(true)
            .justify(gtk4::Justification::Center)
            .build();

        let desc_lbl = Label::builder()
            .label(desc)
            .css_classes(vec!["caption"])
            .wrap(true)
            .justify(gtk4::Justification::Center)
            .build();

        content.append(&icon_img);
        content.append(&title_lbl);
        content.append(&desc_lbl);

        button.set_child(Some(&content));

        let stack_clone = stack.clone();
        let page_name = target_page.to_string();
        button.connect_clicked(move |_| {
            stack_clone.set_visible_child_name(&page_name);
        });

        button
    }

    fn create_placeholder(title: &str, stack: &Stack) -> Box {
        let container = Box::new(Orientation::Vertical, 0);
        
        // Header
        let header = Box::new(Orientation::Horizontal, 12);
        header.set_margin_top(12);
        header.set_margin_bottom(12);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let back_btn = Button::builder()
            .icon_name("go-previous-symbolic")
            .build();
        
        let stack_clone = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("dashboard");
        });

        let title_lbl = Label::builder()
            .label(title)
            .css_classes(vec!["title-2"])
            .build();

        header.append(&back_btn);
        header.append(&title_lbl);
        container.append(&header);

        // Content
        let content = Box::new(Orientation::Vertical, 0);
        content.set_valign(Align::Center);
        content.set_halign(Align::Center);
        content.set_vexpand(true);

        content.append(&Label::new(Some("This tool is under construction.")));
        
        container.append(&content);
        container
    }
}

struct TabbedBoxWidgets {
    width: Entry,
    depth: Entry,
    height: Entry,
    outside: CheckButton,
    thickness: Entry,
    burn: Entry,
    finger_width: Entry,
    space_width: Entry,
    surrounding_spaces: Entry,
    play: Entry,
    extra_length: Entry,
    // New controls
    box_type: ComboBoxText,
    dividers_x: Entry,
    dividers_y: Entry,
    divider_keying: ComboBoxText,
    optimize_layout: CheckButton,
    passes: Entry,
    power: Entry,
    feed_rate: Entry,
    offset_x: Entry,
    offset_y: Entry,
}

pub struct TabbedBoxMaker {
    pub content: Box,
}

impl TabbedBoxMaker {
    pub fn new<F: Fn(String) + 'static>(stack: &Stack, on_generate: F) -> Self {
        let content_box = Box::new(Orientation::Vertical, 0);
        
        // Header with Back Button
        let header = Box::new(Orientation::Horizontal, 12);
        header.set_margin_top(12);
        header.set_margin_bottom(12);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let back_btn = Button::builder()
            .icon_name("go-previous-symbolic")
            .build();
        
        let stack_clone = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("dashboard");
        });

        let title_lbl = Label::builder()
            .label("Tabbed Box Maker (boxes.py algorithm)")
            .css_classes(vec!["title-2"])
            .build();

        header.append(&back_btn);
        header.append(&title_lbl);
        content_box.append(&header);

        // Scrollable Content
        let scroll_content = Box::new(Orientation::Vertical, 0);
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&scroll_content)
            .build();

        // Widgets
        let width = Entry::builder().text("100").valign(Align::Center).build();
        let depth = Entry::builder().text("100").valign(Align::Center).build();
        let height = Entry::builder().text("100").valign(Align::Center).build();
        let outside = CheckButton::builder().active(false).valign(Align::Center).build();
        let thickness = Entry::builder().text("3").valign(Align::Center).build();
        let burn = Entry::builder().text("0.1").valign(Align::Center).build();
        let finger_width = Entry::builder().text("2").valign(Align::Center).build();
        let space_width = Entry::builder().text("2").valign(Align::Center).build();
        let surrounding_spaces = Entry::builder().text("2").valign(Align::Center).build();
        let play = Entry::builder().text("0").valign(Align::Center).build();
        let extra_length = Entry::builder().text("0").valign(Align::Center).build();

        // New Widgets
        let box_type = ComboBoxText::new();
        box_type.append(Some("0"), "Full Box");
        box_type.append(Some("1"), "No Top");
        box_type.append(Some("2"), "No Bottom");
        box_type.append(Some("3"), "No Sides");
        box_type.append(Some("4"), "No Front/Back");
        box_type.append(Some("5"), "No Left/Right");
        box_type.set_active_id(Some("0"));
        box_type.set_valign(Align::Center);

        let dividers_x = Entry::builder().text("0").valign(Align::Center).build();
        let dividers_y = Entry::builder().text("0").valign(Align::Center).build();
        
        let divider_keying = ComboBoxText::new();
        divider_keying.append(Some("0"), "Walls & Floor");
        divider_keying.append(Some("1"), "Walls Only");
        divider_keying.append(Some("2"), "Floor Only");
        divider_keying.append(Some("3"), "None");
        divider_keying.set_active_id(Some("0"));
        divider_keying.set_valign(Align::Center);

        let optimize_layout = CheckButton::builder().active(false).valign(Align::Center).build();
        
        let passes = Entry::builder().text("3").valign(Align::Center).build();
        let power = Entry::builder().text("1000").valign(Align::Center).build();
        let feed_rate = Entry::builder().text("500").valign(Align::Center).build();
        
        let offset_x = Entry::builder().text("10").valign(Align::Center).build();
        let offset_y = Entry::builder().text("10").valign(Align::Center).build();

        // Box Dimensions
        let dim_group = PreferencesGroup::builder().title("Box Dimensions (mm)").build();
        dim_group.add(&Self::create_row("X (Width):", &width));
        dim_group.add(&Self::create_row("Y (Depth):", &depth));
        dim_group.add(&Self::create_row("H (Height):", &height));
        
        let outside_row = ActionRow::builder().title("Outside Dims:").build();
        outside_row.add_suffix(&outside);
        dim_group.add(&outside_row);
        
        scroll_content.append(&dim_group);

        // Box Configuration
        let config_group = PreferencesGroup::builder().title("Box Configuration").build();
        config_group.add(&Self::create_row("Box Type:", &box_type));
        config_group.add(&Self::create_row("Dividers X:", &dividers_x));
        config_group.add(&Self::create_row("Dividers Y:", &dividers_y));
        config_group.add(&Self::create_row("Divider Keying:", &divider_keying));
        
        let optimize_row = ActionRow::builder().title("Optimize Layout:").build();
        optimize_row.add_suffix(&optimize_layout);
        config_group.add(&optimize_row);
        
        scroll_content.append(&config_group);

        // Material Settings
        let mat_group = PreferencesGroup::builder().title("Material Settings").build();
        mat_group.add(&Self::create_row("Thickness (mm):", &thickness));
        mat_group.add(&Self::create_row("Burn / Tool Dia (mm):", &burn));
        scroll_content.append(&mat_group);

        // Finger Joint Settings
        let finger_group = PreferencesGroup::builder().title("Finger Joint Settings (multiples of thickness)").build();
        finger_group.add(&Self::create_row("Finger Width:", &finger_width));
        finger_group.add(&Self::create_row("Space Width:", &space_width));
        finger_group.add(&Self::create_row("Surrounding Spaces:", &surrounding_spaces));
        finger_group.add(&Self::create_row("Play (fit tolerance):", &play));
        finger_group.add(&Self::create_row("Extra Length:", &extra_length));
        scroll_content.append(&finger_group);

        // Laser Settings
        let laser_group = PreferencesGroup::builder().title("Laser Settings").build();
        laser_group.add(&Self::create_row("Passes:", &passes));
        laser_group.add(&Self::create_row("Power (S):", &power));
        laser_group.add(&Self::create_row("Feed Rate:", &feed_rate));
        scroll_content.append(&laser_group);

        // Work Origin Offsets
        let offset_group = PreferencesGroup::builder().title("Work Origin Offsets (mm)").build();
        offset_group.add(&Self::create_row("Offset X:", &offset_x));
        offset_group.add(&Self::create_row("Offset Y:", &offset_y));
        scroll_content.append(&offset_group);

        content_box.append(&scrolled);

        // Action Buttons
        let action_box = Box::new(Orientation::Horizontal, 12);
        action_box.set_margin_top(12);
        action_box.set_margin_bottom(12);
        action_box.set_margin_end(12);
        action_box.set_halign(Align::End);

        let load_btn = Button::with_label("Load");
        let save_btn = Button::with_label("Save");
        let cancel_btn = Button::with_label("Cancel");
        let generate_btn = Button::with_label("Generate");
        generate_btn.add_css_class("suggested-action");

        action_box.append(&load_btn);
        action_box.append(&save_btn);
        action_box.append(&cancel_btn);
        action_box.append(&generate_btn);
        content_box.append(&action_box);

        let widgets = Rc::new(TabbedBoxWidgets {
            width, depth, height, outside, thickness, burn,
            finger_width, space_width, surrounding_spaces, play, extra_length,
            box_type, dividers_x, dividers_y, divider_keying, optimize_layout,
            passes, power, feed_rate, offset_x, offset_y,
        });

        // Connect Signals
        let widgets_gen = widgets.clone();
        let on_generate = Rc::new(on_generate);
        generate_btn.connect_clicked(move |_| {
            let params = Self::collect_params(&widgets_gen);
            match Generator::new(params) {
                Ok(mut generator) => {
                    if let Ok(_) = generator.generate() {
                        let gcode = generator.to_gcode();
                        on_generate(gcode);
                    } else {
                        eprintln!("Failed to generate box paths");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to initialize generator: {}", e);
                }
            }
        });

        let widgets_save = widgets.clone();
        save_btn.connect_clicked(move |_| {
            let params = Self::collect_params(&widgets_save);
            Self::save_params(&params);
        });

        let widgets_load = widgets.clone();
        load_btn.connect_clicked(move |_| {
            Self::load_params(&widgets_load);
        });

        let stack_clone_cancel = stack.clone();
        cancel_btn.connect_clicked(move |_| {
            stack_clone_cancel.set_visible_child_name("dashboard");
        });

        Self { content: content_box }
    }

    pub fn widget(&self) -> &Box {
        &self.content
    }

    fn create_row(title: &str, widget: &impl IsA<gtk4::Widget>) -> ActionRow {
        let row = ActionRow::builder().title(title).build();
        row.add_suffix(widget);
        row
    }

    fn collect_params(w: &TabbedBoxWidgets) -> BoxParameters {
        let mut params = BoxParameters::default();
        
        params.x = w.width.text().parse().unwrap_or(100.0);
        params.y = w.depth.text().parse().unwrap_or(100.0);
        params.h = w.height.text().parse().unwrap_or(100.0);
        params.outside = w.outside.is_active();
        params.thickness = w.thickness.text().parse().unwrap_or(3.0);
        params.burn = w.burn.text().parse().unwrap_or(0.1);
        
        params.finger_joint.finger = w.finger_width.text().parse().unwrap_or(2.0);
        params.finger_joint.space = w.space_width.text().parse().unwrap_or(2.0);
        params.finger_joint.surrounding_spaces = w.surrounding_spaces.text().parse().unwrap_or(2.0);
        params.finger_joint.play = w.play.text().parse().unwrap_or(0.0);
        params.finger_joint.extra_length = w.extra_length.text().parse().unwrap_or(0.0);

        // New params
        if let Some(id) = w.box_type.active_id() {
            params.box_type = BoxType::from(id.parse::<i32>().unwrap_or(0));
        }
        params.dividers_x = w.dividers_x.text().parse().unwrap_or(0);
        params.dividers_y = w.dividers_y.text().parse().unwrap_or(0);
        if let Some(id) = w.divider_keying.active_id() {
            params.key_divider_type = KeyDividerType::from(id.parse::<i32>().unwrap_or(0));
        }
        params.optimize_layout = w.optimize_layout.is_active();
        
        params.laser_passes = w.passes.text().parse().unwrap_or(3);
        params.laser_power = w.power.text().parse().unwrap_or(1000);
        params.feed_rate = w.feed_rate.text().parse().unwrap_or(500.0);
        
        params.offset_x = w.offset_x.text().parse().unwrap_or(10.0);
        params.offset_y = w.offset_y.text().parse().unwrap_or(10.0);

        params
    }

    fn save_params(params: &BoxParameters) {
        let dialog = FileChooserDialog::new(
            Some("Save Box Parameters"),
            None::<&gtk4::Window>,
            FileChooserAction::Save,
            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
        );
        
        dialog.set_current_name("box_params.json");

        let params_clone = params.clone();
        dialog.connect_response(move |d, response| {
            if response == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        if let Ok(json) = serde_json::to_string_pretty(&params_clone) {
                            let _ = fs::write(path, json);
                        }
                    }
                }
            }
            d.close();
        });
        
        dialog.show();
    }

    fn load_params(w: &Rc<TabbedBoxWidgets>) {
        let dialog = FileChooserDialog::new(
            Some("Load Box Parameters"),
            None::<&gtk4::Window>,
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );

        let w_clone = w.clone();
        dialog.connect_response(move |d, response| {
            if response == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        if let Ok(content) = fs::read_to_string(path) {
                            if let Ok(params) = serde_json::from_str::<BoxParameters>(&content) {
                                Self::apply_params(&w_clone, &params);
                            }
                        }
                    }
                }
            }
            d.close();
        });
        
        dialog.show();
    }

    fn apply_params(w: &TabbedBoxWidgets, p: &BoxParameters) {
        w.width.set_text(&p.x.to_string());
        w.depth.set_text(&p.y.to_string());
        w.height.set_text(&p.h.to_string());
        w.outside.set_active(p.outside);
        w.thickness.set_text(&p.thickness.to_string());
        w.burn.set_text(&p.burn.to_string());
        
        w.finger_width.set_text(&p.finger_joint.finger.to_string());
        w.space_width.set_text(&p.finger_joint.space.to_string());
        w.surrounding_spaces.set_text(&p.finger_joint.surrounding_spaces.to_string());
        w.play.set_text(&p.finger_joint.play.to_string());
        w.extra_length.set_text(&p.finger_joint.extra_length.to_string());

        // New params
        w.box_type.set_active_id(Some(&(p.box_type as i32).to_string()));
        w.dividers_x.set_text(&p.dividers_x.to_string());
        w.dividers_y.set_text(&p.dividers_y.to_string());
        w.divider_keying.set_active_id(Some(&(p.key_divider_type as i32).to_string()));
        w.optimize_layout.set_active(p.optimize_layout);
        
        w.passes.set_text(&p.laser_passes.to_string());
        w.power.set_text(&p.laser_power.to_string());
        w.feed_rate.set_text(&p.feed_rate.to_string());
        
        w.offset_x.set_text(&p.offset_x.to_string());
        w.offset_y.set_text(&p.offset_y.to_string());
    }
}
