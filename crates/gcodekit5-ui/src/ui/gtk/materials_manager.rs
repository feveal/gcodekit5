use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, ComboBoxText, Entry, Frame, Grid, Label, ListBox, Orientation,
    Paned, PolicyType, ScrolledWindow, SearchEntry, SpinButton, Stack, StackSwitcher, TextView,
    WrapMode,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::materials_manager_backend::MaterialsManagerBackend;
use gcodekit5_core::data::materials::{Material, MaterialId};

#[derive(Clone)]
pub struct MaterialsManagerView {
    pub widget: Paned,
    backend: Rc<RefCell<MaterialsManagerBackend>>,
    materials_list: ListBox,
    search_entry: SearchEntry,
    category_filter: ComboBoxText,

    // Edit form widgets
    edit_id: Entry,
    edit_name: Entry,
    edit_category: ComboBoxText,
    edit_subcategory: Entry,
    edit_description: TextView,
    edit_density: SpinButton,
    edit_machinability: SpinButton,
    edit_tensile_strength: Entry,
    edit_melting_point: Entry,
    edit_chip_type: ComboBoxText,
    edit_heat_sensitivity: ComboBoxText,
    edit_abrasiveness: ComboBoxText,
    edit_surface_finish: ComboBoxText,
    edit_dust_hazard: ComboBoxText,
    edit_fume_hazard: ComboBoxText,
    edit_coolant_required: CheckButton,
    edit_notes: TextView,

    // State
    selected_material: Rc<RefCell<Option<Material>>>,
    is_creating: Rc<RefCell<bool>>,

    // Action buttons
    save_btn: Button,
    cancel_btn: Button,
    delete_btn: Button,
    new_btn: Button,
}

impl MaterialsManagerView {
    pub fn new() -> Rc<Self> {
        let backend = Rc::new(RefCell::new(MaterialsManagerBackend::new()));

        let widget = Paned::new(Orientation::Horizontal);
        widget.set_hexpand(true);
        widget.set_vexpand(true);

        // ═══════════════════════════════════════════════
        // LEFT SIDEBAR - Materials List
        // ═══════════════════════════════════════════════
        let sidebar = Box::new(Orientation::Vertical, 10);
        sidebar.add_css_class("sidebar");
        sidebar.set_width_request(250);
        sidebar.set_margin_top(10);
        sidebar.set_margin_bottom(10);
        sidebar.set_margin_start(10);
        sidebar.set_margin_end(10);

        // Header
        let header_box = Box::new(Orientation::Horizontal, 10);
        let title = Label::new(Some("Materials"));
        title.add_css_class("title-4");
        title.set_halign(Align::Start);
        header_box.append(&title);
        sidebar.append(&header_box);

        // Search
        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search materials..."));
        sidebar.append(&search_entry);

        // Category filter
        let category_filter = ComboBoxText::new();
        category_filter.append(Some("all"), "All Categories");
        category_filter.append(Some("wood"), "Wood");
        category_filter.append(Some("eng_wood"), "Engineered Wood");
        category_filter.append(Some("plastic"), "Plastic");
        category_filter.append(Some("non_ferrous"), "Non-Ferrous Metal");
        category_filter.append(Some("ferrous"), "Ferrous Metal");
        category_filter.append(Some("composite"), "Composite");
        category_filter.append(Some("stone"), "Stone & Ceramic");
        category_filter.set_active(Some(0));
        sidebar.append(&category_filter);

        // Materials list
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_vexpand(true);

        let materials_list = ListBox::new();
        materials_list.add_css_class("boxed-list");
        scroll.set_child(Some(&materials_list));
        sidebar.append(&scroll);

        // New material button
        let new_btn = Button::with_label("➕ New Material");
        new_btn.add_css_class("suggested-action");
        sidebar.append(&new_btn);

        widget.set_start_child(Some(&sidebar));

        // ═══════════════════════════════════════════════
        // RIGHT PANEL - Material Details/Edit Form
        // ═══════════════════════════════════════════════
        let main_content = Box::new(Orientation::Vertical, 10);
        main_content.set_margin_top(20);
        main_content.set_margin_bottom(20);
        main_content.set_margin_start(20);
        main_content.set_margin_end(20);

        // Action buttons bar
        let action_bar = Box::new(Orientation::Horizontal, 10);
        let save_btn = Button::with_label("💾 Save");
        save_btn.add_css_class("suggested-action");
        save_btn.set_sensitive(false);
        let cancel_btn = Button::with_label("❌ Cancel");
        cancel_btn.set_sensitive(false);
        let delete_btn = Button::with_label("🗑️ Delete");
        delete_btn.add_css_class("destructive-action");
        delete_btn.set_sensitive(false);

        action_bar.append(&save_btn);
        action_bar.append(&cancel_btn);
        action_bar.append(&delete_btn);

        let spacer = Label::new(None);
        spacer.set_hexpand(true);
        action_bar.append(&spacer);

        main_content.append(&action_bar);

        // Stack with tabs
        let stack = Stack::new();
        stack.set_vexpand(true);

        // Create tab pages
        let (general_page, edit_id, edit_name, edit_category, edit_subcategory, edit_description) =
            Self::create_general_tab();
        let (
            properties_page,
            edit_density,
            edit_machinability,
            edit_tensile_strength,
            edit_melting_point,
        ) = Self::create_properties_tab();
        let (
            machining_page,
            edit_chip_type,
            edit_heat_sensitivity,
            edit_abrasiveness,
            edit_surface_finish,
        ) = Self::create_machining_tab();
        let (safety_page, edit_dust_hazard, edit_fume_hazard, edit_coolant_required) =
            Self::create_safety_tab();
        let (notes_page, edit_notes) = Self::create_notes_tab();

        stack.add_titled(&general_page, Some("general"), "Basic Info");
        stack.add_titled(&properties_page, Some("properties"), "Properties");
        stack.add_titled(&machining_page, Some("machining"), "Machining");
        stack.add_titled(&safety_page, Some("safety"), "Safety");
        stack.add_titled(&notes_page, Some("notes"), "Notes");

        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_halign(Align::Center);

        main_content.append(&switcher);
        main_content.append(&stack);

        widget.set_end_child(Some(&main_content));

        // Set initial position
        widget.add_tick_callback(|paned, _clock| {
            let width = paned.width();
            let target = (width as f64 * 0.2) as i32;
            if (paned.position() - target).abs() > 2 {
                paned.set_position(target);
            }
            gtk4::glib::ControlFlow::Continue
        });

        let view = Rc::new(Self {
            widget,
            backend: backend.clone(),
            materials_list,
            search_entry,
            category_filter,
            edit_id,
            edit_name,
            edit_category,
            edit_subcategory,
            edit_description,
            edit_density,
            edit_machinability,
            edit_tensile_strength,
            edit_melting_point,
            edit_chip_type,
            edit_heat_sensitivity,
            edit_abrasiveness,
            edit_surface_finish,
            edit_dust_hazard,
            edit_fume_hazard,
            edit_coolant_required,
            edit_notes,
            selected_material: Rc::new(RefCell::new(None)),
            is_creating: Rc::new(RefCell::new(false)),
            save_btn,
            cancel_btn,
            delete_btn,
            new_btn,
        });

        view.setup_event_handlers();
        view.load_materials();

        view
    }

    fn create_general_tab() -> (ScrolledWindow, Entry, Entry, ComboBoxText, Entry, TextView) {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

        let vbox = Box::new(Orientation::Vertical, 15);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        // ID (shown only when creating)
        let id_grid = Grid::new();
        id_grid.set_column_spacing(10);
        id_grid.set_row_spacing(5);
        let id_label = Label::new(Some("ID:"));
        id_label.set_halign(Align::Start);
        let edit_id = Entry::new();
        edit_id.set_placeholder_text(Some("unique_id"));
        id_grid.attach(&id_label, 0, 0, 1, 1);
        id_grid.attach(&edit_id, 1, 0, 1, 1);
        vbox.append(&id_grid);

        // Name
        let name_grid = Grid::new();
        name_grid.set_column_spacing(10);
        name_grid.set_row_spacing(5);
        let name_label = Label::new(Some("Name:"));
        name_label.set_halign(Align::Start);
        let edit_name = Entry::new();
        edit_name.set_hexpand(true);
        name_grid.attach(&name_label, 0, 0, 1, 1);
        name_grid.attach(&edit_name, 1, 0, 1, 1);
        vbox.append(&name_grid);

        // Category
        let cat_grid = Grid::new();
        cat_grid.set_column_spacing(10);
        cat_grid.set_row_spacing(5);
        let cat_label = Label::new(Some("Category:"));
        cat_label.set_halign(Align::Start);
        let edit_category = ComboBoxText::new();
        edit_category.append_text("Wood");
        edit_category.append_text("Engineered Wood");
        edit_category.append_text("Plastic");
        edit_category.append_text("Non-Ferrous Metal");
        edit_category.append_text("Ferrous Metal");
        edit_category.append_text("Composite");
        edit_category.append_text("Stone & Ceramic");
        edit_category.set_active(Some(0));
        cat_grid.attach(&cat_label, 0, 0, 1, 1);
        cat_grid.attach(&edit_category, 1, 0, 1, 1);
        vbox.append(&cat_grid);

        // Subcategory
        let subcat_grid = Grid::new();
        subcat_grid.set_column_spacing(10);
        subcat_grid.set_row_spacing(5);
        let subcat_label = Label::new(Some("Subcategory:"));
        subcat_label.set_halign(Align::Start);
        let edit_subcategory = Entry::new();
        edit_subcategory.set_placeholder_text(Some("e.g., Hardwood, Alloy"));
        edit_subcategory.set_hexpand(true);
        subcat_grid.attach(&subcat_label, 0, 0, 1, 1);
        subcat_grid.attach(&edit_subcategory, 1, 0, 1, 1);
        vbox.append(&subcat_grid);

        // Description
        let desc_frame = Frame::new(Some("Description"));
        let edit_description = TextView::new();
        edit_description.set_wrap_mode(WrapMode::Word);
        edit_description.set_height_request(80);
        let desc_scroll = ScrolledWindow::new();
        desc_scroll.set_child(Some(&edit_description));
        desc_frame.set_child(Some(&desc_scroll));
        vbox.append(&desc_frame);

        scroll.set_child(Some(&vbox));
        (
            scroll,
            edit_id,
            edit_name,
            edit_category,
            edit_subcategory,
            edit_description,
        )
    }

    fn create_properties_tab() -> (ScrolledWindow, SpinButton, SpinButton, Entry, Entry) {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

        let grid = Grid::new();
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        grid.set_margin_start(10);
        grid.set_margin_end(10);
        grid.set_column_spacing(10);
        grid.set_row_spacing(10);

        let mut row = 0;

        // Density
        let density_label = Label::new(Some("Density (kg/m³):"));
        density_label.set_halign(Align::Start);
        let edit_density = SpinButton::with_range(0.0, 20000.0, 1.0);
        edit_density.set_value(750.0);
        edit_density.set_hexpand(true);
        grid.attach(&density_label, 0, row, 1, 1);
        grid.attach(&edit_density, 1, row, 1, 1);
        row += 1;

        // Machinability
        let mach_label = Label::new(Some("Machinability (1-10):"));
        mach_label.set_halign(Align::Start);
        let edit_machinability = SpinButton::with_range(1.0, 10.0, 1.0);
        edit_machinability.set_value(7.0);
        grid.attach(&mach_label, 0, row, 1, 1);
        grid.attach(&edit_machinability, 1, row, 1, 1);
        row += 1;

        // Tensile strength
        let tens_label = Label::new(Some("Tensile Strength (MPa):"));
        tens_label.set_halign(Align::Start);
        let edit_tensile_strength = Entry::new();
        edit_tensile_strength.set_placeholder_text(Some("Optional"));
        grid.attach(&tens_label, 0, row, 1, 1);
        grid.attach(&edit_tensile_strength, 1, row, 1, 1);
        row += 1;

        // Melting point
        let melt_label = Label::new(Some("Melting Point (°C):"));
        melt_label.set_halign(Align::Start);
        let edit_melting_point = Entry::new();
        edit_melting_point.set_placeholder_text(Some("Optional"));
        grid.attach(&melt_label, 0, row, 1, 1);
        grid.attach(&edit_melting_point, 1, row, 1, 1);

        scroll.set_child(Some(&grid));
        (
            scroll,
            edit_density,
            edit_machinability,
            edit_tensile_strength,
            edit_melting_point,
        )
    }

    fn create_machining_tab() -> (
        ScrolledWindow,
        ComboBoxText,
        ComboBoxText,
        ComboBoxText,
        ComboBoxText,
    ) {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

        let grid = Grid::new();
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        grid.set_margin_start(10);
        grid.set_margin_end(10);
        grid.set_column_spacing(10);
        grid.set_row_spacing(10);

        let mut row = 0;

        // Chip type
        let chip_label = Label::new(Some("Chip Type:"));
        chip_label.set_halign(Align::Start);
        let edit_chip_type = ComboBoxText::new();
        edit_chip_type.append_text("Continuous");
        edit_chip_type.append_text("Segmented");
        edit_chip_type.append_text("Granular");
        edit_chip_type.append_text("Small");
        edit_chip_type.set_active(Some(0));
        grid.attach(&chip_label, 0, row, 1, 1);
        grid.attach(&edit_chip_type, 1, row, 1, 1);
        row += 1;

        // Heat sensitivity
        let heat_label = Label::new(Some("Heat Sensitivity:"));
        heat_label.set_halign(Align::Start);
        let edit_heat_sensitivity = ComboBoxText::new();
        edit_heat_sensitivity.append_text("Low");
        edit_heat_sensitivity.append_text("Moderate");
        edit_heat_sensitivity.append_text("High");
        edit_heat_sensitivity.set_active(Some(0));
        grid.attach(&heat_label, 0, row, 1, 1);
        grid.attach(&edit_heat_sensitivity, 1, row, 1, 1);
        row += 1;

        // Abrasiveness
        let abr_label = Label::new(Some("Abrasiveness:"));
        abr_label.set_halign(Align::Start);
        let edit_abrasiveness = ComboBoxText::new();
        edit_abrasiveness.append_text("Low");
        edit_abrasiveness.append_text("Moderate");
        edit_abrasiveness.append_text("High");
        edit_abrasiveness.set_active(Some(0));
        grid.attach(&abr_label, 0, row, 1, 1);
        grid.attach(&edit_abrasiveness, 1, row, 1, 1);
        row += 1;

        // Surface finish
        let surf_label = Label::new(Some("Surface Finish:"));
        surf_label.set_halign(Align::Start);
        let edit_surface_finish = ComboBoxText::new();
        edit_surface_finish.append_text("Excellent");
        edit_surface_finish.append_text("Good");
        edit_surface_finish.append_text("Fair");
        edit_surface_finish.append_text("Rough");
        edit_surface_finish.set_active(Some(1));
        grid.attach(&surf_label, 0, row, 1, 1);
        grid.attach(&edit_surface_finish, 1, row, 1, 1);

        scroll.set_child(Some(&grid));
        (
            scroll,
            edit_chip_type,
            edit_heat_sensitivity,
            edit_abrasiveness,
            edit_surface_finish,
        )
    }

    fn create_safety_tab() -> (ScrolledWindow, ComboBoxText, ComboBoxText, CheckButton) {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

        let grid = Grid::new();
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        grid.set_margin_start(10);
        grid.set_margin_end(10);
        grid.set_column_spacing(10);
        grid.set_row_spacing(10);

        let mut row = 0;

        // Dust hazard
        let dust_label = Label::new(Some("Dust Hazard:"));
        dust_label.set_halign(Align::Start);
        let edit_dust_hazard = ComboBoxText::new();
        edit_dust_hazard.append_text("None");
        edit_dust_hazard.append_text("Minimal");
        edit_dust_hazard.append_text("Moderate");
        edit_dust_hazard.append_text("High");
        edit_dust_hazard.set_active(Some(1));
        grid.attach(&dust_label, 0, row, 1, 1);
        grid.attach(&edit_dust_hazard, 1, row, 1, 1);
        row += 1;

        // Fume hazard
        let fume_label = Label::new(Some("Fume Hazard:"));
        fume_label.set_halign(Align::Start);
        let edit_fume_hazard = ComboBoxText::new();
        edit_fume_hazard.append_text("None");
        edit_fume_hazard.append_text("Minimal");
        edit_fume_hazard.append_text("Moderate");
        edit_fume_hazard.append_text("High");
        edit_fume_hazard.set_active(Some(0));
        grid.attach(&fume_label, 0, row, 1, 1);
        grid.attach(&edit_fume_hazard, 1, row, 1, 1);
        row += 1;

        // Coolant required
        let edit_coolant_required = CheckButton::with_label("Coolant Required");
        grid.attach(&edit_coolant_required, 0, row, 2, 1);

        scroll.set_child(Some(&grid));
        (
            scroll,
            edit_dust_hazard,
            edit_fume_hazard,
            edit_coolant_required,
        )
    }

    fn create_notes_tab() -> (ScrolledWindow, TextView) {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        let label = Label::new(Some("Additional Notes and Tips:"));
        label.set_halign(Align::Start);
        vbox.append(&label);

        let edit_notes = TextView::new();
        edit_notes.set_wrap_mode(WrapMode::Word);
        edit_notes.set_vexpand(true);
        vbox.append(&edit_notes);

        scroll.set_child(Some(&vbox));
        (scroll, edit_notes)
    }

    fn setup_event_handlers(self: &Rc<Self>) {
        // New material button
        let view = self.clone();
        self.new_btn.connect_clicked(move |_| {
            view.start_create_new();
        });

        // Save button
        let view = self.clone();
        self.save_btn.connect_clicked(move |_| {
            view.save_material();
        });

        // Cancel button
        let view = self.clone();
        self.cancel_btn.connect_clicked(move |_| {
            view.cancel_edit();
        });

        // Delete button
        let view = self.clone();
        self.delete_btn.connect_clicked(move |_| {
            view.delete_material();
        });

        // Search
        let view = self.clone();
        self.search_entry.connect_search_changed(move |_| {
            view.load_materials();
        });

        // Category filter
        let view = self.clone();
        self.category_filter.connect_changed(move |_| {
            view.load_materials();
        });

        // List selection
        let view = self.clone();
        self.materials_list.connect_row_activated(move |_, row| {
            // Get the Box widget from the row
            if let Some(row_box) = row.child().and_then(|w| w.downcast::<Box>().ok()) {
                // Find the hidden label containing the material ID (last child)
                let mut child = row_box.first_child();
                let mut id_label: Option<Label> = None;

                while let Some(widget) = child.clone() {
                    if let Ok(label) = widget.clone().downcast::<Label>() {
                        if !label.is_visible() {
                            id_label = Some(label.clone());
                            break;
                        }
                    }
                    child = widget.next_sibling();
                }

                if let Some(label) = id_label {
                    let material_id = label.label().to_string();
                    view.load_material_for_edit(&material_id);
                }
            }
        });
    }

    fn load_materials(&self) {
        // Clear list
        while let Some(child) = self.materials_list.first_child() {
            self.materials_list.remove(&child);
        }

        let backend = self.backend.borrow();
        let search_query = self.search_entry.text().to_string();
        let materials = backend.search_materials(&search_query);

        for material in materials {
            let row_box = Box::new(Orientation::Vertical, 5);
            row_box.set_margin_top(5);
            row_box.set_margin_bottom(5);
            row_box.set_margin_start(10);
            row_box.set_margin_end(10);

            let name_label = Label::new(Some(&material.name));
            name_label.add_css_class("title-4");
            name_label.set_halign(Align::Start);
            row_box.append(&name_label);

            let info = format!("{} - {}", material.category, material.subcategory);
            let info_label = Label::new(Some(&info));
            info_label.add_css_class("dim-label");
            info_label.set_halign(Align::Start);
            row_box.append(&info_label);

            let mach_info = format!("Machinability: {}/10", material.machinability_rating);
            let mach_label = Label::new(Some(&mach_info));
            mach_label.set_halign(Align::Start);
            row_box.append(&mach_label);

            // Store material ID as widget data
            let id_label = Label::new(Some(&material.id.0));
            id_label.set_visible(false);
            row_box.append(&id_label);

            self.materials_list.append(&row_box);
        }
    }

    fn start_create_new(&self) {
        *self.is_creating.borrow_mut() = true;
        *self.selected_material.borrow_mut() = None;

        self.clear_form();
        self.edit_id.set_sensitive(true);
        self.save_btn.set_sensitive(true);
        self.cancel_btn.set_sensitive(true);
        self.delete_btn.set_sensitive(false);
    }

    fn load_material_for_edit(&self, material_id: &str) {
        let backend = self.backend.borrow();
        let mat_id = MaterialId(material_id.to_string());

        if let Some(material) = backend.get_material(&mat_id) {
            *self.is_creating.borrow_mut() = false;
            *self.selected_material.borrow_mut() = Some(material.clone());

            // Load material into form
            self.edit_id.set_text(&material.id.0);
            self.edit_name.set_text(&material.name);

            // Set category
            let category_text = material.category.to_string();
            for i in 0..7 {
                if let Some(text) = self.edit_category.active_text() {
                    if text == category_text {
                        break;
                    }
                }
                self.edit_category.set_active(Some(i));
            }

            self.edit_subcategory.set_text(&material.subcategory);
            self.edit_description
                .buffer()
                .set_text(&material.description);
            self.edit_density.set_value(material.density as f64);
            self.edit_machinability
                .set_value(material.machinability_rating as f64);

            if let Some(ts) = material.tensile_strength {
                self.edit_tensile_strength.set_text(&ts.to_string());
            } else {
                self.edit_tensile_strength.set_text("");
            }

            if let Some(mp) = material.melting_point {
                self.edit_melting_point.set_text(&mp.to_string());
            } else {
                self.edit_melting_point.set_text("");
            }

            // Set chip type
            let chip_text = format!("{:?}", material.chip_type);
            for i in 0..4 {
                if let Some(text) = self.edit_chip_type.active_text() {
                    if text == chip_text {
                        break;
                    }
                }
                self.edit_chip_type.set_active(Some(i));
            }

            // Set heat sensitivity
            let heat_text = format!("{:?}", material.heat_sensitivity);
            for i in 0..3 {
                if let Some(text) = self.edit_heat_sensitivity.active_text() {
                    if text == heat_text {
                        break;
                    }
                }
                self.edit_heat_sensitivity.set_active(Some(i));
            }

            // Set abrasiveness
            let abr_text = format!("{:?}", material.abrasiveness);
            for i in 0..3 {
                if let Some(text) = self.edit_abrasiveness.active_text() {
                    if text == abr_text {
                        break;
                    }
                }
                self.edit_abrasiveness.set_active(Some(i));
            }

            // Set surface finish
            let surf_text = format!("{:?}", material.surface_finish);
            for i in 0..4 {
                if let Some(text) = self.edit_surface_finish.active_text() {
                    if text == surf_text {
                        break;
                    }
                }
                self.edit_surface_finish.set_active(Some(i));
            }

            // Set dust hazard
            let dust_text = format!("{:?}", material.dust_hazard);
            for i in 0..4 {
                if let Some(text) = self.edit_dust_hazard.active_text() {
                    if text == dust_text {
                        break;
                    }
                }
                self.edit_dust_hazard.set_active(Some(i));
            }

            // Set fume hazard
            let fume_text = format!("{:?}", material.fume_hazard);
            for i in 0..4 {
                if let Some(text) = self.edit_fume_hazard.active_text() {
                    if text == fume_text {
                        break;
                    }
                }
                self.edit_fume_hazard.set_active(Some(i));
            }

            self.edit_coolant_required
                .set_active(material.coolant_required);
            self.edit_notes.buffer().set_text(&material.notes);

            // Update UI state
            self.edit_id.set_sensitive(false);
            self.save_btn.set_sensitive(true);
            self.cancel_btn.set_sensitive(true);
            self.delete_btn.set_sensitive(material.custom);
        }
    }

    fn save_material(&self) {
        // TODO: Collect form data and save
        self.load_materials();
        self.cancel_edit();
    }

    fn delete_material(&self) {
        // TODO: Delete selected material
        self.load_materials();
        self.cancel_edit();
    }

    fn cancel_edit(&self) {
        *self.is_creating.borrow_mut() = false;
        *self.selected_material.borrow_mut() = None;
        self.clear_form();
        self.save_btn.set_sensitive(false);
        self.cancel_btn.set_sensitive(false);
        self.delete_btn.set_sensitive(false);
    }

    fn clear_form(&self) {
        self.edit_id.set_text("");
        self.edit_name.set_text("");
        self.edit_category.set_active(Some(0));
        self.edit_subcategory.set_text("");
        self.edit_description.buffer().set_text("");
        self.edit_density.set_value(750.0);
        self.edit_machinability.set_value(7.0);
        self.edit_tensile_strength.set_text("");
        self.edit_melting_point.set_text("");
        self.edit_chip_type.set_active(Some(0));
        self.edit_heat_sensitivity.set_active(Some(0));
        self.edit_abrasiveness.set_active(Some(0));
        self.edit_surface_finish.set_active(Some(1));
        self.edit_dust_hazard.set_active(Some(1));
        self.edit_fume_hazard.set_active(Some(0));
        self.edit_coolant_required.set_active(false);
        self.edit_notes.buffer().set_text("");
    }
}
