use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Separator, Stack, Widget};
use std::cell::RefCell;
use std::rc::Rc;

pub struct DeviceInfoView {
    pub container: Box,
}

impl DeviceInfoView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Sidebar (approx 20% width, similar to other views)
        let sidebar = Box::new(Orientation::Vertical, 10);
        sidebar.set_width_request(250);
        sidebar.set_margin_top(15); sidebar.set_margin_bottom(15); sidebar.set_margin_start(15); sidebar.set_margin_end(15);;
        sidebar.add_css_class("sidebar");

        // Status Icon
        let status_icon = Label::new(Some("🔌"));
        status_icon.set_css_classes(&["status-icon-large"]);
        status_icon.set_height_request(100);
        sidebar.append(&status_icon);

        // Device Info
        Self::add_info_row(&sidebar, "DEVICE NAME", "No Device");
        Self::add_info_row(&sidebar, "FIRMWARE", "-");
        Self::add_info_row(&sidebar, "VERSION", "-");

        sidebar.append(&Separator::new(Orientation::Horizontal));

        // Actions
        let refresh_btn = Button::with_label("Refresh Info");
        refresh_btn.set_icon_name("view-refresh-symbolic");
        sidebar.append(&refresh_btn);

        let copy_btn = Button::with_label("Copy Config");
        copy_btn.set_icon_name("edit-copy-symbolic");
        sidebar.append(&copy_btn);

        container.append(&sidebar);
        container.append(&Separator::new(Orientation::Vertical));

        // Main Content
        let main_content = Box::new(Orientation::Vertical, 15);
        main_content.set_hexpand(true);
        main_content.set_vexpand(true);
        main_content.set_margin_top(20); main_content.set_margin_bottom(20); main_content.set_margin_start(20); main_content.set_margin_end(20);;

        let title = Label::new(Some("Firmware Capabilities"));
        title.set_css_classes(&["title-2"]);
        title.set_halign(gtk4::Align::Start);
        main_content.append(&title);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        
        let capabilities_list = Box::new(Orientation::Vertical, 2);
        scroll.set_child(Some(&capabilities_list));
        main_content.append(&scroll);

        // Placeholder for capabilities
        let placeholder = Label::new(Some("Connect a device to view capabilities"));
        placeholder.set_css_classes(&["text-muted"]);
        capabilities_list.append(&placeholder);

        container.append(&main_content);

        Self { container }
    }

    fn add_info_row(container: &Box, label: &str, value: &str) {
        let vbox = Box::new(Orientation::Vertical, 5);
        let lbl = Label::new(Some(label));
        lbl.set_css_classes(&["label-small", "text-muted", "font-bold"]);
        lbl.set_halign(gtk4::Align::Start);
        vbox.append(&lbl);

        let val = Label::new(Some(value));
        val.set_css_classes(&["label-medium", "font-bold"]);
        val.set_halign(gtk4::Align::Start);
        vbox.append(&val);

        container.append(&vbox);
    }
}
