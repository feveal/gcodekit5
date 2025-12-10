use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, Image, Label, Expander, Align, ScrolledWindow, PolicyType, Entry};
use std::cell::RefCell;
use std::rc::Rc;
use gcodekit5_designer::designer_state::DesignerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignerTool {
    Select = 0,
    Rectangle = 1,
    Circle = 2,
    Line = 3,
    Ellipse = 4,
    Polyline = 5,
    Text = 6,
    Pan = 7,
}

impl DesignerTool {
    pub fn name(&self) -> &'static str {
        match self {
            DesignerTool::Select => "Select",
            DesignerTool::Rectangle => "Rectangle",
            DesignerTool::Circle => "Circle",
            DesignerTool::Line => "Line",
            DesignerTool::Ellipse => "Ellipse",
            DesignerTool::Polyline => "Polyline",
            DesignerTool::Text => "Text",
            DesignerTool::Pan => "Pan",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            DesignerTool::Select => "select.svg",
            DesignerTool::Rectangle => "rectangle.svg",
            DesignerTool::Circle => "circle.svg",
            DesignerTool::Line => "line.svg",
            DesignerTool::Ellipse => "ellipse.svg",
            DesignerTool::Polyline => "polyline.svg",
            DesignerTool::Text => "text.svg",
            DesignerTool::Pan => "grab.svg",
        }
    }
    
    pub fn tooltip(&self) -> &'static str {
        match self {
            DesignerTool::Select => "Select and move shapes (S)",
            DesignerTool::Rectangle => "Draw rectangle (R)",
            DesignerTool::Circle => "Draw circle (C)",
            DesignerTool::Line => "Draw line (L)",
            DesignerTool::Ellipse => "Draw ellipse (E)",
            DesignerTool::Polyline => "Draw polyline/polygon (P)",
            DesignerTool::Text => "Add text (T)",
            DesignerTool::Pan => "Pan canvas (Space)",
        }
    }
}

pub struct DesignerToolbox {
    pub widget: Box,
    current_tool: Rc<RefCell<DesignerTool>>,
    buttons: Vec<Button>,
    tools: Vec<DesignerTool>,
    generate_btn: Button,
    _state: Rc<RefCell<DesignerState>>,
}

impl DesignerToolbox {
    pub fn new(state: Rc<RefCell<DesignerState>>) -> Rc<Self> {
        let main_container = Box::new(Orientation::Vertical, 0);
        main_container.set_width_request(160); // Increased width for 3 columns
        main_container.set_hexpand(false);
        main_container.add_css_class("designer-toolbox");
        main_container.set_margin_top(5);
        main_container.set_margin_bottom(5);
        main_container.set_margin_start(5);
        main_container.set_margin_end(5);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(false)
            .vexpand(true)
            .build();

        let content_box = Box::new(Orientation::Vertical, 2);
        
        let current_tool = Rc::new(RefCell::new(DesignerTool::Select));
        let mut buttons: Vec<Button> = Vec::new();
        
        let tools = vec![
            DesignerTool::Select,
            DesignerTool::Pan,
            DesignerTool::Rectangle,
            DesignerTool::Circle,
            DesignerTool::Line,
            DesignerTool::Ellipse,
            DesignerTool::Polyline,
            DesignerTool::Text,
        ];
        
        let grid = gtk4::Grid::builder()
            .column_spacing(2)
            .row_spacing(2)
            .halign(Align::Center)
            .build();
        
        for (i, tool) in tools.iter().enumerate() {
            let btn = Button::new();
            btn.set_size_request(40, 40); // Slightly smaller for 3 columns
            btn.set_halign(Align::Center);
            btn.set_tooltip_text(Some(tool.tooltip()));
            
            // Use icon from compiled resources
            let icon_filename = tool.icon();
            let resource_path = format!("/com/gcodekit5/icons/{}", icon_filename);
            
            let icon = Image::from_resource(&resource_path);
            icon.set_pixel_size(20);
            btn.set_child(Some(&icon));
            
            buttons.push(btn.clone());
            
            // Select tool is initially selected
            if *tool == DesignerTool::Select {
                btn.add_css_class("selected-tool");
            }
            
            grid.attach(&btn, (i % 3) as i32, (i / 3) as i32, 1, 1);
        }
        
        content_box.append(&grid);
        
        // Now wire up click handlers after all buttons are collected
        for (i, btn) in buttons.iter().enumerate() {
            let current_tool_clone = current_tool.clone();
            let buttons_clone = buttons.clone();
            let tools_clone = tools.clone();
            let tool = tools[i];
            
            btn.connect_clicked(move |_| {
                *current_tool_clone.borrow_mut() = tool;
                // Update button styles
                for (j, b) in buttons_clone.iter().enumerate() {
                    if tools_clone[j] == tool {
                        b.add_css_class("selected-tool");
                    } else {
                        b.remove_css_class("selected-tool");
                    }
                }
            });
        }
        
        // Add separator
        let separator = gtk4::Separator::new(Orientation::Horizontal);
        separator.set_margin_top(10);
        separator.set_margin_bottom(10);
        content_box.append(&separator);

        // Tool Settings
        let settings_box = Box::new(Orientation::Vertical, 5);
        settings_box.set_margin_start(2);
        settings_box.set_margin_end(2);

        // Helper to create labeled entry
        let create_setting = |label_text: &str, value: f64, tooltip: &str| -> Entry {
            let label = Label::builder()
                .label(label_text)
                .halign(Align::Start)
                .build();
            label.add_css_class("small-label");
            settings_box.append(&label);

            let entry = Entry::builder()
                .text(&format!("{:.2}", value))
                .tooltip_text(tooltip)
                .build();
            settings_box.append(&entry);
            entry
        };

        let current_settings = state.borrow().tool_settings.clone();

        // Feed Rate
        let feed_entry = create_setting("Feed (mm/min)", current_settings.feed_rate, "Feed Rate");
        let state_feed = state.clone();
        feed_entry.connect_changed(move |entry| {
            if let Ok(val) = entry.text().parse::<f64>() {
                state_feed.borrow_mut().set_feed_rate(val);
            }
        });

        // Spindle Speed
        let speed_entry = create_setting("Speed (RPM)", current_settings.spindle_speed as f64, "Spindle Speed");
        let state_speed = state.clone();
        speed_entry.connect_changed(move |entry| {
            if let Ok(val) = entry.text().parse::<f64>() {
                state_speed.borrow_mut().set_spindle_speed(val as u32);
            }
        });

        // Tool Diameter
        let diam_entry = create_setting("Tool Dia (mm)", current_settings.tool_diameter, "Tool Diameter");
        let state_diam = state.clone();
        diam_entry.connect_changed(move |entry| {
            if let Ok(val) = entry.text().parse::<f64>() {
                state_diam.borrow_mut().set_tool_diameter(val);
            }
        });

        // Cut Depth
        let depth_entry = create_setting("Cut Depth (mm)", current_settings.cut_depth, "Target Cut Depth (positive)");
        let state_depth = state.clone();
        depth_entry.connect_changed(move |entry| {
            if let Ok(val) = entry.text().parse::<f64>() {
                state_depth.borrow_mut().set_cut_depth(val);
            }
        });

        // Step Down
        let step_entry = create_setting("Step Down (mm)", current_settings.step_down as f64, "Depth per pass");
        let state_step = state.clone();
        step_entry.connect_changed(move |entry| {
            if let Ok(val) = entry.text().parse::<f64>() {
                state_step.borrow_mut().set_step_down(val);
            }
        });

        let expander = Expander::builder()
            .label("Tool Settings")
            .child(&settings_box)
            .expanded(true)
            .build();
        
        content_box.append(&expander);

        // Generate G-Code Button
        let generate_btn = Button::with_label("Generate G-Code");
        generate_btn.add_css_class("suggested-action");
        generate_btn.set_margin_top(10);
        generate_btn.set_margin_bottom(10);
        generate_btn.set_margin_start(5);
        generate_btn.set_margin_end(5);
        content_box.append(&generate_btn);

        scrolled.set_child(Some(&content_box));
        main_container.append(&scrolled);
        
        Rc::new(Self {
            widget: main_container,
            current_tool,
            buttons,
            tools,
            generate_btn,
            _state: state,
        })
    }
    
    pub fn connect_generate_clicked<F: Fn() + 'static>(&self, f: F) {
        self.generate_btn.connect_clicked(move |_| f());
    }
    
    pub fn current_tool(&self) -> DesignerTool {
        *self.current_tool.borrow()
    }
    
    pub fn set_tool(&self, tool: DesignerTool) {
        *self.current_tool.borrow_mut() = tool;
        
        // Update button styles
        for (i, btn) in self.buttons.iter().enumerate() {
            if self.tools[i] == tool {
                btn.add_css_class("selected-tool");
            } else {
                btn.remove_css_class("selected-tool");
            }
        }
    }
}
