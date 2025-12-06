use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, ComboBoxText, Entry, Label, ListView, Orientation,
    ScrolledWindow, SelectionMode, SignalListItemFactory, SingleSelection, StringList, Window,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage, PreferencesWindow};
use std::cell::RefCell;
use std::rc::Rc;

use gcodekit5_settings::controller::{SettingUiModel, SettingsController};
use gcodekit5_settings::view_model::SettingsCategory;

pub struct SettingsWindow {
    window: PreferencesWindow,
    controller: Rc<SettingsController>,
}

impl SettingsWindow {
    pub fn new(controller: Rc<SettingsController>) -> Self {
        let window = PreferencesWindow::builder()
            .title("Preferences")
            .modal(true)
            .default_width(800)
            .default_height(600)
            .build();

        let settings_window = Self { window, controller };
        settings_window.setup_pages();
        settings_window
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn setup_pages(&self) {
        // General Page
        self.add_page(SettingsCategory::General, "General", "system-run-symbolic");
        
        // Controller Page
        self.add_page(SettingsCategory::Controller, "Controller", "input-gaming-symbolic");
        
        // UI Page
        self.add_page(SettingsCategory::UserInterface, "User Interface", "preferences-desktop-display-symbolic");
        
        // File Processing Page
        self.add_page(SettingsCategory::FileProcessing, "File Processing", "document-open-symbolic");
        
        // Shortcuts Page
        self.add_page(SettingsCategory::KeyboardShortcuts, "Shortcuts", "input-keyboard-symbolic");
        
        // Advanced Page
        self.add_page(SettingsCategory::Advanced, "Advanced", "preferences-system-symbolic");
    }

    fn add_page(&self, category: SettingsCategory, title: &str, icon_name: &str) {
        let page = PreferencesPage::builder()
            .title(title)
            .icon_name(icon_name)
            .build();

        let group = PreferencesGroup::builder()
            .title(title)
            .build();

        let settings = self.controller.get_settings_for_ui(Some(category));
        
        for setting in settings {
            let row = self.create_setting_row(&setting);
            group.add(&row);
        }

        page.add(&group);
        self.window.add(&page);
    }

    fn create_setting_row(&self, setting: &SettingUiModel) -> ActionRow {
        let row = ActionRow::builder()
            .title(&setting.name)
            .subtitle(&setting.description)
            .build();

        let controller = self.controller.clone();
        let id = setting.id.clone();

        match setting.value_type.as_str() {
            "Boolean" => {
                let switch = CheckButton::builder()
                    .active(setting.value == "true")
                    .valign(Align::Center)
                    .build();
                
                let id_clone = id.clone();
                let controller_clone = controller.clone();
                let window_clone = self.window.clone();
                switch.connect_toggled(move |btn| {
                    controller_clone.update_setting(&id_clone, &btn.is_active().to_string());
                    if let Err(e) = controller_clone.save() {
                        eprintln!("Failed to save settings: {}", e);
                    } else {
                        // Optional: Show toast?
                        // window_clone.add_toast(libadwaita::Toast::new("Setting saved"));
                    }
                });
                
                row.add_suffix(&switch);
                row.set_activatable_widget(Some(&switch));
            },
            "Enum" => {
                let combo = ComboBoxText::new();
                for (i, option) in setting.options.iter().enumerate() {
                    combo.append(Some(option), option);
                    if i as i32 == setting.current_index {
                        combo.set_active(Some(i as u32));
                    }
                }
                
                let id_clone = id.clone();
                let controller_clone = controller.clone();
                combo.connect_changed(move |c| {
                    if let Some(text) = c.active_text() {
                        controller_clone.update_setting(&id_clone, &text);
                        let _ = controller_clone.save();
                    }
                });
                
                row.add_suffix(&combo);
            },
            "Path" => {
                let entry = Entry::builder()
                    .text(&setting.value)
                    .valign(Align::Center)
                    .width_chars(20)
                    .build();
                
                let browse_btn = Button::builder()
                    .icon_name("folder-open-symbolic")
                    .valign(Align::Center)
                    .build();
                
                // TODO: Implement file chooser dialog for browse button
                
                let id_clone = id.clone();
                let controller_clone = controller.clone();
                entry.connect_changed(move |e| {
                    controller_clone.update_setting(&id_clone, &e.text());
                    let _ = controller_clone.save();
                });

                let box_container = Box::new(Orientation::Horizontal, 6);
                box_container.append(&entry);
                box_container.append(&browse_btn);
                
                row.add_suffix(&box_container);
            },
            _ => { // String, Integer, Float
                let entry = Entry::builder()
                    .text(&setting.value)
                    .valign(Align::Center)
                    .width_chars(20)
                    .build();
                
                let id_clone = id.clone();
                let controller_clone = controller.clone();
                entry.connect_changed(move |e| {
                    controller_clone.update_setting(&id_clone, &e.text());
                    let _ = controller_clone.save();
                });
                
                row.add_suffix(&entry);
            }
        }

        row
    }
}
