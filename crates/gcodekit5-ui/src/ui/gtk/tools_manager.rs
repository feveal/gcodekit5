use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Paned, ScrolledWindow, SearchEntry, Separator, Stack, StackSwitcher, Widget};

pub struct ToolsManagerView {
    pub container: Box,
}

impl ToolsManagerView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let paned = Paned::new(Orientation::Horizontal);
        paned.set_hexpand(true);
        paned.set_vexpand(true);
        paned.set_position(250); // Sidebar width

        // Sidebar
        let sidebar = Box::new(Orientation::Vertical, 10);
        sidebar.set_margin_top(10); sidebar.set_margin_bottom(10); sidebar.set_margin_start(10); sidebar.set_margin_end(10);;
        
        let search = SearchEntry::new();
        search.set_placeholder_text(Some("Search tools..."));
        sidebar.append(&search);

        let tools_list = ScrolledWindow::new();
        tools_list.set_vexpand(true);
        // Placeholder list
        let list_box = Box::new(Orientation::Vertical, 5);
        list_box.append(&Label::new(Some("Tool 1")));
        list_box.append(&Label::new(Some("Tool 2")));
        tools_list.set_child(Some(&list_box));
        sidebar.append(&tools_list);

        let add_btn = Button::with_label("Add Tool");
        add_btn.set_icon_name("list-add-symbolic");
        sidebar.append(&add_btn);

        paned.set_start_child(Some(&sidebar));

        // Main Content
        let main_content = Box::new(Orientation::Vertical, 10);
        main_content.set_margin_top(20); main_content.set_margin_bottom(20); main_content.set_margin_start(20); main_content.set_margin_end(20);;

        let stack = Stack::new();
        stack.set_vexpand(true);

        // Basic Info Tab
        let basic_info = Box::new(Orientation::Vertical, 10);
        basic_info.append(&Label::new(Some("Basic Information Form Placeholder")));
        stack.add_titled(&basic_info, Some("basic"), "Basic Info");

        // Geometry Tab
        let geometry = Box::new(Orientation::Vertical, 10);
        geometry.append(&Label::new(Some("Geometry Form Placeholder")));
        stack.add_titled(&geometry, Some("geometry"), "Geometry");

        // Manufacturer Tab
        let manufacturer = Box::new(Orientation::Vertical, 10);
        manufacturer.append(&Label::new(Some("Manufacturer Form Placeholder")));
        stack.add_titled(&manufacturer, Some("manufacturer"), "Manufacturer");

        // Notes Tab
        let notes = Box::new(Orientation::Vertical, 10);
        notes.append(&Label::new(Some("Notes Form Placeholder")));
        stack.add_titled(&notes, Some("notes"), "Notes");

        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_halign(gtk4::Align::Center);

        main_content.append(&switcher);
        main_content.append(&stack);

        paned.set_end_child(Some(&main_content));
        container.append(&paned);

        Self { container }
    }
}
