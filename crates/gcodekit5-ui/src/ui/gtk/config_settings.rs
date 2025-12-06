use gtk4::prelude::*;
use gtk4::{Box, Button, ColumnView, ColumnViewColumn, Label, Orientation, ScrolledWindow, SearchEntry, SignalListItemFactory, SingleSelection, Widget};
use gtk4::gio;

pub struct ConfigSettingsView {
    pub container: Box,
}

impl ConfigSettingsView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 10);
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_margin_top(10); container.set_margin_bottom(10); container.set_margin_start(10); container.set_margin_end(10);

        // Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 10);
        
        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Filter settings..."));
        search_entry.set_hexpand(true);
        toolbar.append(&search_entry);

        let reload_btn = Button::with_label("Reload");
        reload_btn.set_icon_name("view-refresh-symbolic");
        toolbar.append(&reload_btn);

        let save_btn = Button::with_label("Save to File");
        save_btn.set_icon_name("document-save-symbolic");
        toolbar.append(&save_btn);

        let restore_btn = Button::with_label("Restore");
        restore_btn.set_icon_name("document-revert-symbolic");
        toolbar.append(&restore_btn);

        container.append(&toolbar);

        // Settings List
        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);

        // Note: In a real implementation, we would use a ListStore or similar model.
        // For now, we'll just create the ColumnView structure.
        let column_view = ColumnView::new(None::<gtk4::SingleSelection>);
        
        Self::add_column(&column_view, "ID", true);
        Self::add_column(&column_view, "Name", true);
        Self::add_column(&column_view, "Value", true);
        Self::add_column(&column_view, "Unit", true);
        Self::add_column(&column_view, "Category", true);
        Self::add_column(&column_view, "Description", true);

        scroll.set_child(Some(&column_view));
        container.append(&scroll);

        // Status Bar
        let status_bar = Box::new(Orientation::Horizontal, 10);
        let status_label = Label::new(Some("Ready"));
        status_bar.append(&status_label);
        container.append(&status_bar);

        Self { container }
    }

    fn add_column(view: &ColumnView, title: &str, expand: bool) {
        let factory = SignalListItemFactory::new();
        // In a real app, we'd bind data here
        let column = ColumnViewColumn::new(Some(title), Some(factory));
        if expand {
            column.set_expand(true);
        }
        view.append_column(&column);
    }
}
