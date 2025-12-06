use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Paned, ScrolledWindow, SearchEntry, Stack, StackSwitcher, Widget};

pub struct MaterialsManagerView {
    pub container: Box,
}

impl MaterialsManagerView {
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
        search.set_placeholder_text(Some("Search materials..."));
        sidebar.append(&search);

        let materials_list = ScrolledWindow::new();
        materials_list.set_vexpand(true);
        // Placeholder list
        let list_box = Box::new(Orientation::Vertical, 5);
        list_box.append(&Label::new(Some("Material 1")));
        list_box.append(&Label::new(Some("Material 2")));
        materials_list.set_child(Some(&list_box));
        sidebar.append(&materials_list);

        let add_btn = Button::with_label("Add Material");
        add_btn.set_icon_name("list-add-symbolic");
        sidebar.append(&add_btn);

        paned.set_start_child(Some(&sidebar));

        // Main Content
        let main_content = Box::new(Orientation::Vertical, 10);
        main_content.set_margin_top(20); main_content.set_margin_bottom(20); main_content.set_margin_start(20); main_content.set_margin_end(20);;

        let stack = Stack::new();
        stack.set_vexpand(true);

        // General Tab
        let general = Box::new(Orientation::Vertical, 10);
        general.append(&Label::new(Some("General Info Form Placeholder")));
        stack.add_titled(&general, Some("general"), "General");

        // Properties Tab
        let properties = Box::new(Orientation::Vertical, 10);
        properties.append(&Label::new(Some("Properties Form Placeholder")));
        stack.add_titled(&properties, Some("properties"), "Properties");

        // Machining Tab
        let machining = Box::new(Orientation::Vertical, 10);
        machining.append(&Label::new(Some("Machining Form Placeholder")));
        stack.add_titled(&machining, Some("machining"), "Machining");

        // Safety Tab
        let safety = Box::new(Orientation::Vertical, 10);
        safety.append(&Label::new(Some("Safety Form Placeholder")));
        stack.add_titled(&safety, Some("safety"), "Safety");

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
