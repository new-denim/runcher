//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::SlotOfDouble;
use qt_widgets::QAction;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::QSpinBox;
use qt_widgets::{QToolButton, q_tool_button::ToolButtonPopupMode};
use qt_widgets::QWidget;
use qt_widgets::QWidgetAction;

use qt_gui::QIcon;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::SlotOfBool;
use qt_core::SlotOfInt;

use anyhow::Result;
use getset::*;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use common_utils::sql::{ParamType, Preset, SQLScript};

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

const VIEW_DEBUG: &str = "ui_templates/actions_groupbox.ui";
const VIEW_RELEASE: &str = "ui/actions_groupbox.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ActionsUI {
    play_button: QPtr<QToolButton>,
    enable_logging_checkbox: QBox<QCheckBox>,
    enable_skip_intro_checkbox: QBox<QCheckBox>,
    remove_trait_limit_checkbox: QBox<QCheckBox>,
    remove_siege_attacker_checkbox: QBox<QCheckBox>,
    enable_translations_combobox: QBox<QComboBox>,
    merge_all_mods_checkbox: QBox<QCheckBox>,
    unit_multiplier_spinbox: QBox<QDoubleSpinBox>,
    universal_rebalancer_combobox: QBox<QComboBox>,
    enable_dev_only_ui_checkbox: QBox<QCheckBox>,
    scripts_container: QBox<QWidget>,
    scripts_to_execute: Arc<RwLock<Vec<(SQLScript, QBox<QCheckBox>)>>>,

    settings_button: QPtr<QToolButton>,
    folders_button: QPtr<QToolButton>,
    open_game_root_folder: QPtr<QAction>,
    open_game_data_folder: QPtr<QAction>,
    open_game_content_folder: QPtr<QAction>,
    open_game_secondary_folder: QPtr<QAction>,
    open_game_config_folder: QPtr<QAction>,
    open_runcher_config_folder: QPtr<QAction>,
    open_runcher_error_folder: QPtr<QAction>,

    copy_load_order_button: QPtr<QToolButton>,
    paste_load_order_button: QPtr<QToolButton>,
    reload_button: QPtr<QToolButton>,
    download_subscribed_mods_button: QPtr<QToolButton>,

    profile_load_button: QPtr<QToolButton>,
    profile_save_button: QPtr<QToolButton>,
    profile_manager_button: QPtr<QToolButton>,
    profile_combobox: QPtr<QComboBox>,
    profile_model: QBox<QStandardItemModel>,

    save_combobox: QPtr<QComboBox>,
    save_model: QBox<QStandardItemModel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ActionsUI {

    pub unsafe fn new_launch_script_option(&self, game_key: &str, icon_key: &str, script: &SQLScript, presets: &[Preset]) -> QBox<QCheckBox> {
        let container = QWidget::new_1a(self.scripts_container());
        let presets_container = QWidget::new_1a(&container);
        let params_container = QWidget::new_1a(&container);
        let presets_layout = create_grid_layout(presets_container.static_upcast());
        let param_layout = create_grid_layout(params_container.static_upcast());
        presets_container.set_enabled(false);
        params_container.set_enabled(false);

        let game_key = game_key.to_owned();

        let script_key = script.metadata().key();
        let script_pretty_name = script.metadata().name();
        let script_params = script.metadata().parameters();

        let settings = settings();
        let mut there_are_presets = false;
        let mut is_preset_selected = false;

        // If we have params, add a small combo widget for selecting possible prefabs.
        if !script_params.is_empty() {
            let label_text = QLabel::from_q_string_q_widget(&qtr("preset"), &presets_container);
            let preset_combo = QComboBox::new_1a(&presets_container);
            preset_combo.set_object_name(&QString::from_std_str(format!("{script_key}_preset_combo")));
            preset_combo.add_item_q_string(&QString::from_std_str("No Preset"));

            presets_layout.add_widget_5a(&label_text, 0, 0, 1, 1);
            presets_layout.add_widget_5a(&preset_combo, 0, 1, 1, 1);

            // If we don't have presets, just hide the widget.
            if presets.is_empty() {
                label_text.set_visible(false);
                preset_combo.set_visible(false);

            } else {
                there_are_presets = true;

                for preset in presets {
                    preset_combo.add_item_q_string(&QString::from_std_str(preset.name()));
                }

                let setting_key = format!("script_to_execute_{}_{}_preset", game_key, script_key);
                let setting = settings.value_1a(&QString::from_std_str(&setting_key));
                if setting.is_valid() {
                    let string = setting.to_string();
                    if !string.is_empty() {
                        preset_combo.set_current_text(&setting.to_string());
                    }
                }

                if preset_combo.current_index() != 0 {
                    is_preset_selected = true;
                } else {
                    set_setting_string(&setting_key, "");
                }

                let params_container_ptr = params_container.as_ptr();
                let preset_combo_ptr = preset_combo.as_ptr();
                preset_combo.current_index_changed().connect(&SlotOfInt::new(&preset_combo, move |value| {
                    params_container_ptr.set_enabled(value == 0);

                    set_setting_string(&setting_key, &preset_combo_ptr.item_text(value).to_std_string());
                }));
            }
        }

        // If we already have a preset set, disable the individual params.
        params_container.set_enabled(!there_are_presets || (there_are_presets && !is_preset_selected));

        for (index, param) in script_params.iter().enumerate() {
            let param_key = param.key();
            let param_type = param.r#type();
            let param_default = param.default_value();
            let param_name = param.name();

            let label_text = QLabel::from_q_string_q_widget(&QString::from_std_str(param_name), &params_container);
            param_layout.add_widget_5a(&label_text, index as i32 + 1, 0, 1, 1);

            let setting = format!("script_to_execute_{}_{}_{}", game_key, script_key, param_key);
            let use_default = !settings.value_1a(&QString::from_std_str(&setting)).is_valid();

            match param_type {
                ParamType::Bool => {
                    let widget = QCheckBox::from_q_widget(&params_container);
                    widget.set_object_name(&QString::from_std_str(format!("{script_key}_{param_key}")));

                    let default_value = param_default.parse::<bool>().unwrap_or_default();
                    if use_default {
                        widget.set_checked(default_value);
                    } else {
                        widget.set_checked(setting_bool(&setting));
                    }

                    param_layout.add_widget_5a(&widget, index as i32 + 1, 1, 1, 1);
                    widget.toggled().connect(&SlotOfBool::new(&params_container, move |state| {
                        set_setting_bool(&setting, state);
                    }));
                },
                ParamType::Integer => {
                    let widget = QSpinBox::new_1a(&params_container);
                    widget.set_object_name(&QString::from_std_str(format!("{script_key}_{param_key}")));

                    let default_value = param_default.parse::<i32>().unwrap_or_default();
                    if use_default {
                        widget.set_value(default_value);
                    } else {
                        widget.set_value(setting_int(&setting));
                    }

                    param_layout.add_widget_5a(&widget, index as i32 + 1, 1, 1, 1);
                    widget.value_changed().connect(&SlotOfInt::new(&params_container, move |value| {
                        set_setting_int(&setting, value);
                    }));
                },
                ParamType::Float => {
                    let widget = QDoubleSpinBox::new_1a(&params_container);
                    widget.set_object_name(&QString::from_std_str(format!("{script_key}_{param_key}")));

                    let default_value = param_default.parse::<f32>().unwrap_or_default();
                    if use_default {
                        widget.set_value(default_value as f64);
                    } else {
                        widget.set_value(setting_f32(&setting) as f64);
                    }

                    param_layout.add_widget_5a(&widget, index as i32 + 1, 1, 1, 1);
                    widget.value_changed().connect(&SlotOfDouble::new(&params_container, move |value| {
                        set_setting_f32(&setting, value as f32);
                    }));
                },
            }
        }


        let checkbox = QCheckBox::from_q_widget(&container);
        let icon = QIcon::from_theme_1a(&QString::from_std_str(icon_key));
        let label_icon = QLabel::from_q_widget(&container);
        label_icon.set_pixmap(&icon.pixmap_2_int(22, 22));
        label_icon.set_maximum_width(22);

        let label_text = QLabel::from_q_string_q_widget(&QString::from_std_str(script_pretty_name.trim()), &container);
        label_text.set_fixed_height(26);

        let label_fill = QLabel::from_q_widget(&container);
        let layout = create_grid_layout(container.static_upcast());

        layout.add_widget_5a(&label_icon, 0, 0, 1, 1);
        layout.add_widget_5a(&label_text, 0, 1, 1, 1);
        layout.add_widget_5a(&label_fill, 0, 2, 1, 1);
        layout.add_widget_5a(&checkbox, 0, 3, 1, 1);
        layout.add_widget_5a(&presets_container, 1, 1, 1, 3);
        layout.add_widget_5a(&params_container, 2, 1, 1, 3);
        layout.set_column_stretch(2, 10);

        // If the script is marked as automatic, hide the UI and automatically check the checkbox.
        if *script.metadata().automatic() {
            label_icon.set_visible(false);
            label_text.set_visible(false);
            label_fill.set_visible(false);
            checkbox.set_visible(false);
            presets_container.set_visible(false);
            params_container.set_visible(false);
        }

        let layout = self.scripts_container().layout().static_downcast::<QGridLayout>();
        layout.add_widget(&container);

        let setting = format!("script_to_execute_{}_{}", game_key, script_key);
        let is_enabled = setting_bool(&setting);

        let script_key = script_key.to_owned();
        checkbox.toggled().connect(&SlotOfBool::new(&checkbox, move |state| {
            presets_container.set_enabled(state);

            // Params only has to be enabled if we don't have presets selected.
            let preset_setting_key = format!("script_to_execute_{}_{}_preset", game_key, script_key);
            let preset_setting = settings.value_1a(&QString::from_std_str(&preset_setting_key)).to_string();

            params_container.set_enabled(!there_are_presets || (there_are_presets && preset_setting.is_empty()));

            set_setting_bool(&setting, state);
        }));

        checkbox.set_checked(is_enabled || *script.metadata().automatic());
        checkbox
    }

    pub unsafe fn new_launch_option(menu: &QBox<QMenu>, text_key: &str, icon_key: &str, base_widget: &QBox<QWidget>, option_widget: &QPtr<QWidget>) {
        let action = QWidgetAction::new(menu);
        let icon = QIcon::from_theme_1a(&QString::from_std_str(icon_key));
        let label_icon = QLabel::from_q_widget(base_widget);
        label_icon.set_pixmap(&icon.pixmap_2_int(22, 22));
        label_icon.set_maximum_width(22);

        let label_text = QLabel::from_q_string_q_widget(&qtr(text_key), base_widget);
        label_text.set_fixed_height(26);

        let label_fill = QLabel::from_q_widget(base_widget);
        let layout = create_grid_layout(base_widget.static_upcast());

        layout.add_widget_5a(&label_icon, 0, 0, 1, 1);
        layout.add_widget_5a(&label_text, 0, 1, 1, 1);
        layout.add_widget_5a(&label_fill, 0, 2, 1, 1);
        layout.add_widget_5a(option_widget, 0, 3, 1, 1);
        layout.set_column_stretch(2, 10);
        action.set_default_widget(base_widget);
        menu.add_action(&action);
    }

    pub unsafe fn update_icons(&self) {
        let enable_logging_icon = QIcon::from_theme_1a(&QString::from_std_str("verb"));
        let enable_skip_intro_icon = QIcon::from_theme_1a(&QString::from_std_str("kdenlive-hide-video"));
        let remove_trait_limit_icon = QIcon::from_theme_1a(&QString::from_std_str("folder-unlocked-symbolic"));
        let remove_siege_attacker_icon = QIcon::from_theme_1a(&QString::from_std_str("folder-unlocked-symbolic"));
        let enable_translations_icon = QIcon::from_theme_1a(&QString::from_std_str("language-chooser"));
        let merge_all_mods_icon = QIcon::from_theme_1a(&QString::from_std_str("merge"));
        let unit_multiplier_icon = QIcon::from_theme_1a(&QString::from_std_str("view-time-schedule-calculus"));
        let universal_rebalancer_icon = QIcon::from_theme_1a(&QString::from_std_str("autocorrection"));
        let enable_dev_only_ui_icon = QIcon::from_theme_1a(&QString::from_std_str("verb"));

        let menu = self.play_button().menu();
        for index in 0..menu.actions().count_0a() {

            if index < 8 {
                let action = menu.actions().value_1a(index);
                let widget_action = action.static_downcast::<QWidgetAction>();
                let widget = widget_action.default_widget();
                let layout = widget.layout().static_downcast::<QGridLayout>();
                let child = layout.item_at_position(0, 0).widget();
                let label = child.static_downcast::<QLabel>();

                match index {
                    0 => label.set_pixmap(&enable_logging_icon.pixmap_2_int(22, 22)),
                    1 => label.set_pixmap(&enable_skip_intro_icon.pixmap_2_int(22, 22)),
                    2 => label.set_pixmap(&remove_trait_limit_icon.pixmap_2_int(22, 22)),
                    3 => label.set_pixmap(&remove_siege_attacker_icon.pixmap_2_int(22, 22)),
                    4 => label.set_pixmap(&enable_translations_icon.pixmap_2_int(22, 22)),
                    5 => label.set_pixmap(&merge_all_mods_icon.pixmap_2_int(22, 22)),
                    6 => label.set_pixmap(&unit_multiplier_icon.pixmap_2_int(22, 22)),
                    7 => label.set_pixmap(&universal_rebalancer_icon.pixmap_2_int(22, 22)),
                    8 => label.set_pixmap(&enable_dev_only_ui_icon.pixmap_2_int(22, 22)),
                    _ => {}
                }
            }
        }
    }

    pub unsafe fn new_launch_option_checkbox(menu: &QBox<QMenu>, text_key: &str, icon_key: &str) -> QBox<QCheckBox> {
        let widget = QWidget::new_1a(menu);
        let checkbox = QCheckBox::from_q_widget(&widget);
        Self::new_launch_option(menu, text_key, icon_key, &widget, &checkbox.static_upcast());
        checkbox
    }

    pub unsafe fn new_launch_option_doublespinbox(menu: &QBox<QMenu>, text_key: &str, icon_key: &str) -> QBox<QDoubleSpinBox> {
        let widget = QWidget::new_1a(menu);
        let spinbox = QDoubleSpinBox::new_1a(&widget);
        Self::new_launch_option(menu, text_key, icon_key, &widget, &spinbox.static_upcast());
        spinbox
    }

    pub unsafe fn new_launch_option_combobox(menu: &QBox<QMenu>, text_key: &str, icon_key: &str) -> QBox<QComboBox> {
        let widget = QWidget::new_1a(menu);
        let combobox = QComboBox::new_1a(&widget);
        Self::new_launch_option(menu, text_key, icon_key, &widget, &combobox.static_upcast());
        combobox
    }

    pub unsafe fn new(parent: &QBox<QWidget>) -> Result<Rc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let play_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "play_button")?;
        let play_menu = QMenu::from_q_widget(&play_button);
        let enable_logging_checkbox = Self::new_launch_option_checkbox(&play_menu, "enable_logging", "verb");
        let enable_skip_intro_checkbox = Self::new_launch_option_checkbox(&play_menu, "enable_skip_intro", "kdenlive-hide-video");
        let remove_trait_limit_checkbox = Self::new_launch_option_checkbox(&play_menu, "remove_trait_limit", "folder-unlocked-symbolic");
        let remove_siege_attacker_checkbox = Self::new_launch_option_checkbox(&play_menu, "remove_siege_attacker", "folder-unlocked-symbolic");
        let enable_translations_combobox = Self::new_launch_option_combobox(&play_menu, "enable_translations", "language-chooser");
        let merge_all_mods_checkbox = Self::new_launch_option_checkbox(&play_menu, "merge_all_mods", "merge");
        let unit_multiplier_spinbox = Self::new_launch_option_doublespinbox(&play_menu, "unit_multiplier", "view-time-schedule-calculus");
        let universal_rebalancer_combobox = Self::new_launch_option_combobox(&play_menu, "universal_rebalancer", "view-time-schedule-calculus");
        let enable_dev_only_ui_checkbox = Self::new_launch_option_checkbox(&play_menu, "enable_dev_only_ui", "verb");
        enable_translations_combobox.set_current_index(0);
        unit_multiplier_spinbox.set_value(1.00);
        universal_rebalancer_combobox.set_current_index(0);

        let scripts_action = QWidgetAction::new(&play_menu);
        let scripts_container = QWidget::new_1a(&play_menu);
        create_grid_layout(scripts_container.static_upcast());
        scripts_action.set_default_widget(&scripts_container);

        play_menu.add_action(&scripts_action);

        play_button.set_menu(play_menu.into_raw_ptr());
        play_button.set_popup_mode(ToolButtonPopupMode::MenuButtonPopup);

        let settings_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "settings_button")?;
        let folders_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "folders_button")?;
        play_button.set_tool_tip(&qtr("launch_game"));
        settings_button.set_tool_tip(&qtr("settings"));
        folders_button.set_tool_tip(&qtr("open_folders"));

        let folders_menu = QMenu::from_q_widget(&folders_button);
        let open_game_root_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_root_folder"));
        let open_game_data_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_data_folder"));
        let open_game_content_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_content_folder"));
        let open_game_secondary_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_secondary_folder"));
        let open_game_config_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_config_folder"));
        let open_runcher_config_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_runcher_config_folder"));
        let open_runcher_error_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_runcher_error_folder"));
        folders_button.set_menu(folders_menu.into_raw_ptr());
        folders_button.set_popup_mode(ToolButtonPopupMode::MenuButtonPopup);

        let copy_load_order_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "copy_load_order_button")?;
        let paste_load_order_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "paste_load_order_button")?;
        let reload_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "reload_button")?;
        let download_subscribed_mods_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "download_subscribed_mods_button")?;
        copy_load_order_button.set_tool_tip(&qtr("copy_load_order"));
        paste_load_order_button.set_tool_tip(&qtr("paste_load_order"));
        reload_button.set_tool_tip(&qtr("reload"));
        download_subscribed_mods_button.set_tool_tip(&qtr("download_subscribed_mods"));

        let profile_load_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_load_button")?;
        let profile_save_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_save_button")?;
        let profile_manager_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_manager_button")?;
        let profile_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "profile_combobox")?;
        let profile_model: QBox<QStandardItemModel> = QStandardItemModel::new_1a(&profile_combobox);
        profile_combobox.set_model(&profile_model);
        profile_combobox.line_edit().set_placeholder_text(&qtr("profile_name"));
        profile_load_button.set_tool_tip(&qtr("load_profile"));
        profile_save_button.set_tool_tip(&qtr("save_profile"));
        profile_manager_button.set_tool_tip(&qtr("profile_manager"));

        let save_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "save_combobox")?;
        let save_model: QBox<QStandardItemModel> = QStandardItemModel::new_1a(&save_combobox);
        save_combobox.set_model(&save_model);

        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let ui = Rc::new(Self {
            play_button,
            enable_logging_checkbox,
            enable_skip_intro_checkbox,
            remove_trait_limit_checkbox,
            remove_siege_attacker_checkbox,
            enable_translations_combobox,
            merge_all_mods_checkbox,
            unit_multiplier_spinbox,
            universal_rebalancer_combobox,
            //universal_balancer_ignored: QToolButton::new_0a();
            enable_dev_only_ui_checkbox,
            scripts_container,
            scripts_to_execute: Arc::new(RwLock::new(vec![])),

            settings_button,
            folders_button,
            open_game_root_folder,
            open_game_data_folder,
            open_game_content_folder,
            open_game_secondary_folder,
            open_game_config_folder,
            open_runcher_config_folder,
            open_runcher_error_folder,

            copy_load_order_button,
            paste_load_order_button,
            reload_button,
            download_subscribed_mods_button,

            profile_load_button,
            profile_save_button,
            profile_manager_button,
            profile_combobox,
            profile_model,

            save_combobox,
            save_model
        });

        Ok(ui)
    }
}
