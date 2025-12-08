use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, Image};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignerTool {
    Select = 0,
    Rectangle = 1,
    Circle = 2,
    Line = 3,
    Ellipse = 4,
    Polyline = 5,
    Text = 6,
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
        }
    }
}

pub struct DesignerToolbox {
    pub widget: Box,
    current_tool: Rc<RefCell<DesignerTool>>,
    buttons: Vec<Button>,
}

impl DesignerToolbox {
    pub fn new() -> Rc<Self> {
        let container = Box::new(Orientation::Vertical, 2);
        container.set_width_request(60);
        container.add_css_class("designer-toolbox");
        container.set_margin_top(5);
        container.set_margin_bottom(5);
        container.set_margin_start(5);
        container.set_margin_end(5);
        
        let current_tool = Rc::new(RefCell::new(DesignerTool::Select));
        let mut buttons: Vec<Button> = Vec::new();
        
        let tools = [
            DesignerTool::Select,
            DesignerTool::Rectangle,
            DesignerTool::Circle,
            DesignerTool::Line,
            DesignerTool::Ellipse,
            DesignerTool::Polyline,
            DesignerTool::Text,
        ];
        
        for tool in tools.iter() {
            let btn = Button::new();
            btn.set_size_request(50, 50);
            btn.set_tooltip_text(Some(tool.tooltip()));
            
            // Use icon from compiled resources
            let icon_filename = tool.icon();
            let resource_path = format!("/com/gcodekit5/icons/{}", icon_filename);
            
            let icon = Image::from_resource(&resource_path);
            icon.set_pixel_size(24);
            btn.set_child(Some(&icon));
            
            // Fallback logic is removed as we expect resources to be present.
            // If we wanted to be safe, we could check if the icon loaded properly, 
            // but Image::from_resource doesn't return a Result.
            
            buttons.push(btn.clone());
            
            // Select tool is initially selected
            if *tool == DesignerTool::Select {
                btn.add_css_class("selected-tool");
            }
            
            container.append(&btn);
        }
        
        // Now wire up click handlers after all buttons are collected
        for (i, btn) in buttons.iter().enumerate() {
            let current_tool_clone = current_tool.clone();
            let buttons_clone = buttons.clone();
            let tool = tools[i];
            
            btn.connect_clicked(move |_| {
                *current_tool_clone.borrow_mut() = tool;
                // Update button styles
                for (j, b) in buttons_clone.iter().enumerate() {
                    if j == tool as usize {
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
        container.append(&separator);
        
        Rc::new(Self {
            widget: container,
            current_tool,
            buttons,
        })
    }
    
    pub fn current_tool(&self) -> DesignerTool {
        *self.current_tool.borrow()
    }
    
    pub fn set_tool(&self, tool: DesignerTool) {
        *self.current_tool.borrow_mut() = tool;
        
        // Update button styles
        for (i, btn) in self.buttons.iter().enumerate() {
            if i == tool as usize {
                btn.add_css_class("selected-tool");
            } else {
                btn.remove_css_class("selected-tool");
            }
        }
        // Update button styles
        for (i, btn) in self.buttons.iter().enumerate() {
            btn.remove_css_class("selected-tool");
            if i == tool as usize {
                btn.add_css_class("selected-tool");
            }
        }
    }
}
