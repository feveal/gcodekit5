use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBox, Orientation, ScrolledWindow, IconSize};
use std::rc::Rc;
use std::cell::RefCell;

use gcodekit5_designer::designer_state::DesignerState;

pub struct LayersPanel {
    pub widget: Box,
    list_box: ListBox,
}

impl LayersPanel {
    pub fn new(state: Rc<RefCell<DesignerState>>) -> Self {
        // Main container
        let widget = Box::new(Orientation::Vertical, 6);
        widget.set_margin_start(6);
        widget.set_margin_end(6);
        widget.set_margin_top(6);
        widget.set_margin_bottom(6);
        widget.set_hexpand(false);
        widget.set_width_request(238); // Fit within 250px parent with margins

        // Header with title and buttons
        let header = Box::new(Orientation::Horizontal, 6);
        let title = Label::new(Some("Layers"));
        title.set_halign(gtk4::Align::Start);
        title.add_css_class("heading");
        header.append(&title);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);

        // Group button
        let group_btn = Button::from_icon_name("object-select-symbolic");
        group_btn.set_tooltip_text(Some("Group Selected (Ctrl+G)"));
        header.append(&group_btn);

        // Ungroup button
        let ungroup_btn = Button::from_icon_name("dialog-question-symbolic");
        ungroup_btn.set_tooltip_text(Some("Ungroup (Ctrl+Shift+G)"));
        header.append(&ungroup_btn);

        widget.append(&header);

        // Scrolled list of shapes
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_height_request(150);

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Multiple);
        scrolled.set_child(Some(&list_box));

        widget.append(&scrolled);

        // Z-order controls
        let z_order_box = Box::new(Orientation::Horizontal, 6);
        
        let bring_front_btn = Button::with_label("⬆⬆");
        bring_front_btn.set_tooltip_text(Some("Bring to Front"));
        z_order_box.append(&bring_front_btn);

        let bring_forward_btn = Button::with_label("⬆");
        bring_forward_btn.set_tooltip_text(Some("Bring Forward"));
        z_order_box.append(&bring_forward_btn);

        let send_backward_btn = Button::with_label("⬇");
        send_backward_btn.set_tooltip_text(Some("Send Backward"));
        z_order_box.append(&send_backward_btn);

        let send_back_btn = Button::with_label("⬇⬇");
        send_back_btn.set_tooltip_text(Some("Send to Back"));
        z_order_box.append(&send_back_btn);

        widget.append(&z_order_box);

        // Connect group button
        let state_clone = state.clone();
        group_btn.connect_clicked(move |_| {
            Self::group_selected_shapes(&state_clone);
        });

        // Connect ungroup button
        let state_clone = state.clone();
        ungroup_btn.connect_clicked(move |_| {
            Self::ungroup_selected_shapes(&state_clone);
        });

        // Connect bring to front
        let state_clone = state.clone();
        bring_front_btn.connect_clicked(move |_| {
            Self::bring_to_front(&state_clone);
        });

        // Connect bring forward
        let state_clone = state.clone();
        bring_forward_btn.connect_clicked(move |_| {
            Self::bring_forward(&state_clone);
        });

        // Connect send backward
        let state_clone = state.clone();
        send_backward_btn.connect_clicked(move |_| {
            Self::send_backward(&state_clone);
        });

        // Connect send to back
        let state_clone = state.clone();
        send_back_btn.connect_clicked(move |_| {
            Self::send_to_back(&state_clone);
        });

        // Connect list selection to shape selection
        let state_clone = state.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let id_str = row.widget_name();
                if let Ok(shape_id) = id_str.as_str().parse::<u64>() {
                    let mut state_mut = state_clone.borrow_mut();
                    let canvas = &mut state_mut.canvas;
                    // Deselect all shapes first
                    canvas.selection_manager.deselect_all(&mut canvas.shape_store);
                    // Select this shape
                    if let Some(obj) = canvas.shape_store.get_mut(shape_id) {
                        obj.selected = true;
                        canvas.selection_manager.set_selected_id(Some(shape_id));
                    }
                }
            }
        });

        Self {
            widget,
            list_box,
        }
    }

    pub fn refresh(&self, state: &Rc<RefCell<DesignerState>>) {
        // Clear existing rows
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        let state_ref = state.borrow();
        
        // Get shapes in draw order from shape_store
        for shape_obj in state_ref.canvas.shape_store.iter() {
            let row_box = Box::new(Orientation::Horizontal, 6);
            row_box.set_margin_start(6);
            row_box.set_margin_end(6);
            row_box.set_margin_top(3);
            row_box.set_margin_bottom(3);

            // Shape type icon/label
            let shape_type = match &shape_obj.shape {
                gcodekit5_designer::shapes::Shape::Rectangle(_) => "Rectangle",
                gcodekit5_designer::shapes::Shape::Circle(_) => "Circle",
                gcodekit5_designer::shapes::Shape::Line(_) => "Line",
                gcodekit5_designer::shapes::Shape::Ellipse(_) => "Ellipse",
                gcodekit5_designer::shapes::Shape::Path(_) => "Path",
                gcodekit5_designer::shapes::Shape::Text(_) => "Text",
            };
            let shape_label = Label::new(Some(&format!("{} #{}", shape_type, shape_obj.id)));
            shape_label.set_halign(gtk4::Align::Start);
            row_box.append(&shape_label);

            // Create a list row and set its name to the shape ID
            let list_row = gtk4::ListBoxRow::new();
            list_row.set_widget_name(&shape_obj.id.to_string());
            list_row.set_child(Some(&row_box));
            
            self.list_box.append(&list_row);
        }
    }

    fn group_selected_shapes(state: &Rc<RefCell<DesignerState>>) {
        // TODO: Implement grouping logic when group support is added
        println!("Group selected shapes - not yet implemented");
    }

    fn ungroup_selected_shapes(state: &Rc<RefCell<DesignerState>>) {
        // TODO: Implement ungrouping logic when group support is added
        println!("Ungroup selected shapes - not yet implemented");
    }

    fn bring_to_front(state: &Rc<RefCell<DesignerState>>) {
        let mut state_mut = state.borrow_mut();
        if let Some(shape_id) = state_mut.canvas.selection_manager.selected_id() {
            state_mut.canvas.shape_store.bring_to_front(shape_id);
        }
    }

    fn bring_forward(state: &Rc<RefCell<DesignerState>>) {
        let mut state_mut = state.borrow_mut();
        if let Some(shape_id) = state_mut.canvas.selection_manager.selected_id() {
            state_mut.canvas.shape_store.bring_forward(shape_id);
        }
    }

    fn send_backward(state: &Rc<RefCell<DesignerState>>) {
        let mut state_mut = state.borrow_mut();
        if let Some(shape_id) = state_mut.canvas.selection_manager.selected_id() {
            state_mut.canvas.shape_store.send_backward(shape_id);
        }
    }

    fn send_to_back(state: &Rc<RefCell<DesignerState>>) {
        let mut state_mut = state.borrow_mut();
        if let Some(shape_id) = state_mut.canvas.selection_manager.selected_id() {
            state_mut.canvas.shape_store.send_to_back(shape_id);
        }
    }
}
