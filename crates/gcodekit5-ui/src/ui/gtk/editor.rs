use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Overlay, PolicyType, ScrolledWindow, FileChooserAction, FileChooserNative, ResponseType};
use sourceview5::prelude::*;
use sourceview5::{Buffer, StyleSchemeManager, View};
use std::cell::RefCell;
use std::path::PathBuf;
use std::fs;
use std::rc::Rc;

pub struct GcodeEditor {
    pub widget: Overlay,
    pub view: View,
    pub buffer: Buffer,
    line_counter_label: Label,
    current_file: Rc<RefCell<Option<PathBuf>>>,
}

impl GcodeEditor {
    pub fn new() -> Self {
        let buffer = Buffer::new(None);
        let view = View::with_buffer(&buffer);
        
        view.set_show_line_numbers(true);
        view.set_monospace(true);
        view.set_highlight_current_line(true);
        view.set_tab_width(4);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_show_right_margin(true);
        view.set_right_margin_position(80);
        
        // Try to set a dark style scheme if available, matching the app's dark theme
        let scheme_manager = StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("kate") { // 'kate' is often available and good for code
            buffer.set_style_scheme(Some(&scheme));
        } else if let Some(scheme) = scheme_manager.scheme("classic") {
            buffer.set_style_scheme(Some(&scheme));
        }

        // TODO: Load G-code language definition
        // let lm = LanguageManager::default();
        // if let Some(lang) = lm.language("gcode") {
        //     buffer.set_language(Some(&lang));
        // }

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .child(&view)
            .build();

        // Create overlay to hold scrolled window and floating panel
        let overlay = Overlay::new();
        overlay.set_child(Some(&scrolled));

        // Create floating line counter panel (bottom right)
        let line_counter_box = Box::new(Orientation::Horizontal, 4);
        line_counter_box.add_css_class("visualizer-osd");
        line_counter_box.set_halign(gtk4::Align::End);
        line_counter_box.set_valign(gtk4::Align::End);
        line_counter_box.set_margin_bottom(20);
        line_counter_box.set_margin_end(20);

        let line_counter_label = Label::builder()
            .label("Line 1 / 1")
            .build();
        line_counter_box.append(&line_counter_label);

        overlay.add_overlay(&line_counter_box);

        let editor = Self {
            widget: overlay,
            view: view.clone(),
            buffer: buffer.clone(),
            line_counter_label: line_counter_label.clone(),
            current_file: Rc::new(RefCell::new(None)),
        };

        // Update line counter when cursor moves
        let label_clone = line_counter_label.clone();
        let buffer_clone = buffer.clone();
        view.connect_move_cursor(move |_, _, _, _| {
            Self::update_line_counter(&buffer_clone, &label_clone);
        });

        // Update line counter when buffer changes
        let label_clone = line_counter_label.clone();
        let buffer_clone = buffer.clone();
        buffer.connect_changed(move |_| {
            Self::update_line_counter(&buffer_clone, &label_clone);
        });

        // Update line counter when cursor position changes (mark-set signal)
        let label_clone = line_counter_label.clone();
        let buffer_clone = buffer.clone();
        buffer.connect_mark_set(move |_, _, mark| {
            // Only update for insert mark (cursor position)
            if mark.name().as_deref() == Some("insert") {
                Self::update_line_counter(&buffer_clone, &label_clone);
            }
        });

        // Initial update
        Self::update_line_counter(&buffer, &line_counter_label);

        editor
    }

    fn update_line_counter(buffer: &Buffer, label: &Label) {
        let total_lines = buffer.line_count();
        let insert_mark = buffer.get_insert();
        let cursor_iter = buffer.iter_at_mark(&insert_mark);
        let current_line = cursor_iter.line() + 1; // Lines are 0-indexed
        label.set_text(&format!("Line {} / {}", current_line, total_lines));
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
        // Move cursor to start (line 1, column 1)
        let mut start_iter = self.buffer.start_iter();
        self.buffer.place_cursor(&start_iter);
        // Scroll to top
        self.view.scroll_to_iter(&mut start_iter, 0.0, false, 0.0, 0.0);
    }

    pub fn get_text(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, true).to_string()
    }
    
    pub fn grab_focus(&self) {
        self.view.grab_focus();
    }
    
    pub fn connect_changed<F: Fn(&Buffer) + 'static>(&self, f: F) {
        self.buffer.connect_changed(f);
    }

    pub fn undo(&self) {
        if self.buffer.can_undo() {
            self.buffer.undo();
        }
    }

    pub fn redo(&self) {
        if self.buffer.can_redo() {
            self.buffer.redo();
        }
    }

    pub fn cut(&self) {
        let clipboard = self.widget.clipboard();
        self.buffer.cut_clipboard(&clipboard, true);
    }

    pub fn copy(&self) {
        let clipboard = self.widget.clipboard();
        self.buffer.copy_clipboard(&clipboard);
    }

    pub fn paste(&self) {
        let clipboard = self.widget.clipboard();
        self.buffer.paste_clipboard(&clipboard, None, true);
    }

    pub fn new_file(&self) {
        self.set_text("");
        *self.current_file.borrow_mut() = None;
    }

    pub fn open_file(&self) {
        let dialog = FileChooserNative::builder()
            .title("Open G-Code File")
            .action(FileChooserAction::Open)
            .modal(true)
            .build();

        if let Some(root) = self.widget.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("G-Code Files"));
        filter.add_pattern("*.gcode");
        filter.add_pattern("*.nc");
        filter.add_pattern("*.gc");
        filter.add_pattern("*.tap");
        dialog.add_filter(&filter);

        let all_filter = gtk4::FileFilter::new();
        all_filter.set_name(Some("All Files"));
        all_filter.add_pattern("*");
        dialog.add_filter(&all_filter);

        let buffer = self.buffer.clone();
        let current_file = self.current_file.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                buffer.set_text(&content);
                                *current_file.borrow_mut() = Some(path);
                                // Move cursor to start
                                let mut start_iter = buffer.start_iter();
                                buffer.place_cursor(&start_iter);
                            }
                            Err(e) => {
                                eprintln!("Error reading file: {}", e);
                                // TODO: Show error dialog
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    pub fn save_file(&self) {
        let current_path = self.current_file.borrow().clone();
        
        if let Some(path) = current_path {
            let start = self.buffer.start_iter();
            let end = self.buffer.end_iter();
            let content = self.buffer.text(&start, &end, true);
            
            if let Err(e) = fs::write(&path, content.as_str()) {
                eprintln!("Error saving file: {}", e);
                // TODO: Show error dialog
            }
        } else {
            self.save_as_file();
        }
    }

    pub fn save_as_file(&self) {
        let dialog = FileChooserNative::builder()
            .title("Save G-Code File")
            .action(FileChooserAction::Save)
            .modal(true)
            .build();

        if let Some(root) = self.widget.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                dialog.set_transient_for(Some(window));
            }
        }

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("G-Code Files"));
        filter.add_pattern("*.gcode");
        filter.add_pattern("*.nc");
        filter.add_pattern("*.gc");
        filter.add_pattern("*.tap");
        dialog.add_filter(&filter);

        let buffer = self.buffer.clone();
        let current_file = self.current_file.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(mut path) = file.path() {
                        // Ensure extension
                        if path.extension().is_none() {
                            path.set_extension("gcode");
                        }
                        
                        let start = buffer.start_iter();
                        let end = buffer.end_iter();
                        let content = buffer.text(&start, &end, true);
                        
                        match fs::write(&path, content.as_str()) {
                            Ok(_) => {
                                *current_file.borrow_mut() = Some(path);
                            }
                            Err(e) => {
                                eprintln!("Error saving file: {}", e);
                                // TODO: Show error dialog
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }
}
