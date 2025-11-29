mod app_config;
mod app_store;
mod auto_launch;
mod claude_mcp;
mod claude_plugin;
mod codex_config;
mod commands;
mod config;
mod database;
mod error;
mod gemini_config; // 新增
mod gemini_mcp;
mod init_status;
mod mcp;
mod prompt;
mod prompt_files;
mod provider;
mod provider_defaults;
mod services;
mod settings;
mod store;
mod usage_script;

pub use app_config::{AppType, McpApps, McpServer, MultiAppConfig};
pub use codex_config::{get_codex_auth_path, get_codex_config_path, write_codex_live_atomic};
pub use commands::*;
pub use config::{get_claude_mcp_path, get_claude_settings_path, read_json_file};
pub use database::Database;
pub use error::AppError;
pub use mcp::{
    import_from_claude, import_from_codex, import_from_gemini, remove_server_from_claude,
    remove_server_from_codex, remove_server_from_gemini, sync_enabled_to_claude,
    sync_enabled_to_codex, sync_enabled_to_gemini, sync_single_server_to_claude,
    sync_single_server_to_codex, sync_single_server_to_gemini,
};
pub use provider::{Provider, ProviderMeta};
pub use services::{
    ConfigService, EndpointLatency, McpService, PromptService, ProviderService, SkillService,
    SpeedtestService,
};
pub use settings::{update_settings, AppSettings};
pub use store::AppState;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use std::sync::Arc;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};
#[cfg(target_os = "macos")]
use tauri::{ActivationPolicy, RunEvent};
use tauri::{Emitter, Manager};

#[derive(Clone, Copy)]
struct TrayTexts {
    show_main: &'static str,
    no_provider_hint: &'static str,
    quit: &'static str,
}

impl TrayTexts {
    fn from_language(language: &str) -> Self {
        match language {
            "en" => Self {
                show_main: "Open main window",
                no_provider_hint: "  (No providers yet, please add them from the main window)",
                quit: "Quit",
            },
            "ja" => Self {
                show_main: "メインウィンドウを開く",
                no_provider_hint:
                    "  (プロバイダーがまだありません。メイン画面から追加してください)",
                quit: "終了",
            },
            _ => Self {
                show_main: "打开主界面",
                no_provider_hint: "  (无供应商，请在主界面添加)",
                quit: "退出",
            },
        }
    }
}

struct TrayAppSection {
    app_type: AppType,
    prefix: &'static str,
    header_id: &'static str,
    empty_id: &'static str,
    header_label: &'static str,
    log_name: &'static str,
}

const TRAY_SECTIONS: [TrayAppSection; 3] = [
    TrayAppSection {
        app_type: AppType::Claude,
        prefix: "claude_",
        header_id: "claude_header",
        empty_id: "claude_empty",
        header_label: "─── Claude ───",
        log_name: "Claude",
    },
    TrayAppSection {
        app_type: AppType::Codex,
        prefix: "codex_",
        header_id: "codex_header",
        empty_id: "codex_empty",
        header_label: "─── Codex ───",
        log_name: "Codex",
    },
    TrayAppSection {
        app_type: AppType::Gemini,
        prefix: "gemini_",
        header_id: "gemini_header",
        empty_id: "gemini_empty",
        header_label: "─── Gemini ───",
        log_name: "Gemini",
    },
];

fn append_provider_section<'a>(
    app: &'a tauri::AppHandle,
    mut menu_builder: MenuBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>>,
    manager: Option<&crate::provider::ProviderManager>,
    section: &TrayAppSection,
    tray_texts: &TrayTexts,
) -> Result<MenuBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>>, AppError> {
    let Some(manager) = manager else {
        return Ok(menu_builder);
    };

    let header = MenuItem::with_id(
        app,
        section.header_id,
        section.header_label,
        false,
        None::<&str>,
    )
    .map_err(|e| AppError::Message(format!("创建{}标题失败: {e}", section.log_name)))?;
    menu_builder = menu_builder.item(&header);

    if manager.providers.is_empty() {
        let empty_hint = MenuItem::with_id(
            app,
            section.empty_id,
            tray_texts.no_provider_hint,
            false,
            None::<&str>,
        )
        .map_err(|e| AppError::Message(format!("创建{}空提示失败: {e}", section.log_name)))?;
        return Ok(menu_builder.item(&empty_hint));
    }

    let mut sorted_providers: Vec<_> = manager.providers.iter().collect();
    sorted_providers.sort_by(|(_, a), (_, b)| {
        match (a.sort_index, b.sort_index) {
            (Some(idx_a), Some(idx_b)) => return idx_a.cmp(&idx_b),
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        match (a.created_at, b.created_at) {
            (Some(time_a), Some(time_b)) => return time_a.cmp(&time_b),
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            _ => {}
        }

        a.name.cmp(&b.name)
    });

    for (id, provider) in sorted_providers {
        let is_current = manager.current == *id;
        let item = CheckMenuItem::with_id(
            app,
            format!("{}{}", section.prefix, id),
            &provider.name,
            true,
            is_current,
            None::<&str>,
        )
        .map_err(|e| AppError::Message(format!("创建{}菜单项失败: {e}", section.log_name)))?;
        menu_builder = menu_builder.item(&item);
    }

    Ok(menu_builder)
}

fn handle_provider_tray_event(app: &tauri::AppHandle, event_id: &str) -> bool {
    for section in TRAY_SECTIONS.iter() {
        if let Some(provider_id) = event_id.strip_prefix(section.prefix) {
            log::info!("切换到{}供应商: {provider_id}", section.log_name);
            let app_handle = app.clone();
            let provider_id = provider_id.to_string();
            let app_type = section.app_type.clone();
            tauri::async_runtime::spawn_blocking(move || {
                if let Err(e) = switch_provider_internal(&app_handle, app_type, provider_id) {
                    log::error!("切换{}供应商失败: {e}", section.log_name);
                }
            });
            return true;
        }
    }
    false
}

/// 创建动态托盘菜单
fn create_tray_menu(
    app: &tauri::AppHandle,
    app_state: &AppState,
) -> Result<Menu<tauri::Wry>, AppError> {
    let app_settings = crate::settings::get_settings();
    let tray_texts = TrayTexts::from_language(app_settings.language.as_deref().unwrap_or("zh"));

    let mut menu_builder = MenuBuilder::new(app);

    // 顶部：打开主界面
    let show_main_item =
        MenuItem::with_id(app, "show_main", tray_texts.show_main, true, None::<&str>)
            .map_err(|e| AppError::Message(format!("创建打开主界面菜单失败: {e}")))?;
    menu_builder = menu_builder.item(&show_main_item).separator();

    // 直接添加所有供应商到主菜单（扁平化结构，更简单可靠）
    for section in TRAY_SECTIONS.iter() {
        let app_type_str = section.app_type.as_str();
        let providers = app_state.db.get_all_providers(app_type_str)?;

        // 使用有效的当前供应商 ID（验证存在性，自动清理失效 ID）
        let current_id =
            crate::settings::get_effective_current_provider(&app_state.db, &section.app_type)?
                .unwrap_or_default();

        let manager = crate::provider::ProviderManager {
            providers,
            current: current_id,
        };

        menu_builder =
            append_provider_section(app, menu_builder, Some(&manager), section, &tray_texts)?;
    }

    // 分隔符和退出菜单
    let quit_item = MenuItem::with_id(app, "quit", tray_texts.quit, true, None::<&str>)
        .map_err(|e| AppError::Message(format!("创建退出菜单失败: {e}")))?;

    menu_builder = menu_builder.separator().item(&quit_item);

    menu_builder
        .build()
        .map_err(|e| AppError::Message(format!("构建菜单失败: {e}")))
}

#[cfg(target_os = "macos")]
fn apply_tray_policy(app: &tauri::AppHandle, dock_visible: bool) {
    let desired_policy = if dock_visible {
        ActivationPolicy::Regular
    } else {
        ActivationPolicy::Accessory
    };

    if let Err(err) = app.set_dock_visibility(dock_visible) {
        log::warn!("设置 Dock 显示状态失败: {err}");
    }

    if let Err(err) = app.set_activation_policy(desired_policy) {
        log::warn!("设置激活策略失败: {err}");
    }
}

/// 处理托盘菜单事件
fn handle_tray_menu_event(app: &tauri::AppHandle, event_id: &str) {
    log::info!("处理托盘菜单事件: {event_id}");

    match event_id {
        "show_main" => {
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    let _ = window.set_skip_taskbar(false);
                }
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
                #[cfg(target_os = "macos")]
                {
                    apply_tray_policy(app, true);
                }
            }
        }
        "quit" => {
            log::info!("退出应用");
            app.exit(0);
        }
        _ => {
            if handle_provider_tray_event(app, event_id) {
                return;
            }
            log::warn!("未处理的菜单事件: {event_id}");
        }
    }
}


/// 内部切换供应商函数
fn switch_provider_internal(
    app: &tauri::AppHandle,
    app_type: crate::app_config::AppType,
    provider_id: String,
) -> Result<(), AppError> {
    if let Some(app_state) = app.try_state::<AppState>() {
        // 在使用前先保存需要的值
        let app_type_str = app_type.as_str().to_string();
        let provider_id_clone = provider_id.clone();

        crate::commands::switch_provider(app_state.clone(), app_type_str.clone(), provider_id)
            .map_err(AppError::Message)?;

        // 切换成功后重新创建托盘菜单
        if let Ok(new_menu) = create_tray_menu(app, app_state.inner()) {
            if let Some(tray) = app.tray_by_id("main") {
                if let Err(e) = tray.set_menu(Some(new_menu)) {
                    log::error!("更新托盘菜单失败: {e}");
                }
            }
        }

        // 发射事件到前端，通知供应商已切换
        let event_data = serde_json::json!({
            "appType": app_type_str,
            "providerId": provider_id_clone
        });
        if let Err(e) = app.emit("provider-switched", event_data) {
            log::error!("发射供应商切换事件失败: {e}");
        }
    }
    Ok(())
}

/// 更新托盘菜单的Tauri命令
#[tauri::command]
async fn update_tray_menu(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    match create_tray_menu(&app, state.inner()) {
        Ok(new_menu) => {
            if let Some(tray) = app.tray_by_id("main") {
                tray.set_menu(Some(new_menu))
                    .map_err(|e| format!("更新托盘菜单失败: {e}"))?;
                return Ok(true);
            }
            Ok(false)
        }
        Err(err) => {
            log::error!("创建托盘菜单失败: {err}");
            Ok(false)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Show and focus window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    let builder = builder
        // 拦截窗口关闭：根据设置决定是否最小化到托盘
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = crate::settings::get_settings();

                if settings.minimize_to_tray_on_close {
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    {
                        let _ = window.set_skip_taskbar(true);
                    }
                    #[cfg(target_os = "macos")]
                    {
                        apply_tray_policy(window.app_handle(), false);
                    }
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // 注册 Updater 插件（桌面端）
            #[cfg(desktop)]
            {
                if let Err(e) = app
                    .handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())
                {
                    // 若配置不完整（如缺少 pubkey），跳过 Updater 而不中断应用
                    log::warn!("初始化 Updater 插件失败，已跳过：{e}");
                }
            }
            #[cfg(target_os = "macos")]
            {
                // 设置 macOS 标题栏背景色为主界面蓝色
                if let Some(window) = app.get_webview_window("main") {
                    use objc2::rc::Retained;
                    use objc2::runtime::AnyObject;
                    use objc2_app_kit::NSColor;

                    match window.ns_window() {
                        Ok(ns_window_ptr) => {
                            if let Some(ns_window) =
                                unsafe { Retained::retain(ns_window_ptr as *mut AnyObject) }
                            {
                                // 使用与主界面 banner 相同的蓝色 #3498db
                                // #3498db = RGB(52, 152, 219)
                                let bg_color = unsafe {
                                    NSColor::colorWithRed_green_blue_alpha(
                                        52.0 / 255.0,  // R: 52
                                        152.0 / 255.0, // G: 152
                                        219.0 / 255.0, // B: 219
                                        1.0,           // Alpha: 1.0
                                    )
                                };

                                unsafe {
                                    use objc2::msg_send;
                                    let _: () =
                                        msg_send![&*ns_window, setBackgroundColor: &*bg_color];
                                }
                            } else {
                                log::warn!("Failed to retain NSWindow reference");
                            }
                        }
                        Err(e) => log::warn!("Failed to get NSWindow pointer: {e}"),
                    }
                }
            }

            // 初始化日志
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 预先刷新 Store 覆盖配置，确保 AppState 初始化时可读取到最新路径
            app_store::refresh_app_config_dir_override(app.handle());

            // 初始化数据库
            let app_config_dir = crate::config::get_app_config_dir();
            let db_path = app_config_dir.join("cc-switch.db");
            let json_path = app_config_dir.join("config.json");

            // 检查是否需要从 config.json 迁移到 SQLite
            let has_json = json_path.exists();
            let has_db = db_path.exists();

            // 如果需要迁移，先验证 config.json 是否可以加载（在创建数据库之前）
            // 这样如果加载失败用户选择退出，数据库文件还没被创建，下次可以正常重试
            let migration_config = if !has_db && has_json {
                log::info!("检测到旧版配置文件，验证配置文件...");

                // 循环：支持用户重试加载配置文件
                loop {
                    match crate::app_config::MultiAppConfig::load() {
                        Ok(config) => {
                            log::info!("✓ 配置文件加载成功");
                            break Some(config);
                        }
                        Err(e) => {
                            log::error!("加载旧配置文件失败: {e}");
                            // 弹出系统对话框让用户选择
                            if !show_migration_error_dialog(app.handle(), &e.to_string()) {
                                // 用户选择退出（此时数据库还没创建，下次启动可以重试）
                                log::info!("用户选择退出程序");
                                std::process::exit(1);
                            }
                            // 用户选择重试，继续循环
                            log::info!("用户选择重试加载配置文件");
                        }
                    }
                }
            } else {
                None
            };

            // 现在创建数据库
            let db = match crate::database::Database::init() {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    log::error!("Failed to init database: {e}");
                    return Err(Box::new(e));
                }
            };

            // 如果有预加载的配置，执行迁移
            if let Some(config) = migration_config {
                log::info!("开始执行数据迁移...");

                match db.migrate_from_json(&config) {
                    Ok(_) => {
                        log::info!("✓ 配置迁移成功");
                        // 标记迁移成功，供前端显示 Toast
                        crate::init_status::set_migration_success();
                        // 归档旧配置文件（重命名而非删除，便于用户恢复）
                        let archive_path = json_path.with_extension("json.migrated");
                        if let Err(e) = std::fs::rename(&json_path, &archive_path) {
                            log::warn!("归档旧配置文件失败: {e}");
                        } else {
                            log::info!("✓ 旧配置已归档为 config.json.migrated");
                        }
                    }
                    Err(e) => {
                        // 配置加载成功但迁移失败的情况极少（磁盘满等），仅记录日志
                        log::error!("配置迁移失败: {e}，将从现有配置导入");
                    }
                }
            }

            let app_state = AppState::new(db);

            // ============================================================
            // 按表独立判断的导入逻辑（各类数据独立检查，互不影响）
            // ============================================================

            // 1. 初始化默认 Skills 仓库（已有内置检查：表非空则跳过）
            match app_state.db.init_default_skill_repos() {
                Ok(count) if count > 0 => {
                    log::info!("✓ Initialized {count} default skill repositories");
                }
                Ok(_) => {} // 表非空，静默跳过
                Err(e) => log::warn!("✗ Failed to initialize default skill repos: {e}"),
            }

            // 2. 导入供应商配置（已有内置检查：该应用已有供应商则跳过）
            for app in [
                crate::app_config::AppType::Claude,
                crate::app_config::AppType::Codex,
                crate::app_config::AppType::Gemini,
            ] {
                match crate::services::provider::ProviderService::import_default_config(
                    &app_state,
                    app.clone(),
                ) {
                    Ok(true) => {
                        log::info!("✓ Imported default provider for {}", app.as_str());
                    }
                    Ok(false) => {} // 已有供应商，静默跳过
                    Err(e) => {
                        log::debug!(
                            "○ No default provider to import for {}: {}",
                            app.as_str(),
                            e
                        );
                    }
                }
            }

            // 3. 导入 MCP 服务器配置（表空时触发）
            if app_state.db.is_mcp_table_empty().unwrap_or(false) {
                log::info!("MCP table empty, importing from live configurations...");

                match crate::services::mcp::McpService::import_from_claude(&app_state) {
                    Ok(count) if count > 0 => {
                        log::info!("✓ Imported {count} MCP server(s) from Claude");
                    }
                    Ok(_) => log::debug!("○ No Claude MCP servers found to import"),
                    Err(e) => log::warn!("✗ Failed to import Claude MCP: {e}"),
                }

                match crate::services::mcp::McpService::import_from_codex(&app_state) {
                    Ok(count) if count > 0 => {
                        log::info!("✓ Imported {count} MCP server(s) from Codex");
                    }
                    Ok(_) => log::debug!("○ No Codex MCP servers found to import"),
                    Err(e) => log::warn!("✗ Failed to import Codex MCP: {e}"),
                }

                match crate::services::mcp::McpService::import_from_gemini(&app_state) {
                    Ok(count) if count > 0 => {
                        log::info!("✓ Imported {count} MCP server(s) from Gemini");
                    }
                    Ok(_) => log::debug!("○ No Gemini MCP servers found to import"),
                    Err(e) => log::warn!("✗ Failed to import Gemini MCP: {e}"),
                }
            }

            // 4. 导入提示词文件（表空时触发）
            if app_state.db.is_prompts_table_empty().unwrap_or(false) {
                log::info!("Prompts table empty, importing from live configurations...");

                for app in [
                    crate::app_config::AppType::Claude,
                    crate::app_config::AppType::Codex,
                    crate::app_config::AppType::Gemini,
                ] {
                    match crate::services::prompt::PromptService::import_from_file_on_first_launch(
                        &app_state,
                        app.clone(),
                    ) {
                        Ok(count) if count > 0 => {
                            log::info!("✓ Imported {count} prompt(s) for {}", app.as_str());
                        }
                        Ok(_) => log::debug!("○ No prompt file found for {}", app.as_str()),
                        Err(e) => log::warn!("✗ Failed to import prompt for {}: {e}", app.as_str()),
                    }
                }
            }

            // 迁移旧的 app_config_dir 配置到 Store
            if let Err(e) = app_store::migrate_app_config_dir_from_settings(app.handle()) {
                log::warn!("迁移 app_config_dir 失败: {e}");
            }

            // 启动阶段不再无条件保存,避免意外覆盖用户配置。

            // 创建动态托盘菜单
            let menu = create_tray_menu(app.handle(), &app_state)?;

            // 构建托盘
            let mut tray_builder = TrayIconBuilder::with_id("main")
                .on_tray_icon_event(|_tray, event| match event {
                    // 左键点击已通过 show_menu_on_left_click(true) 打开菜单，这里不再额外处理
                    TrayIconEvent::Click { .. } => {}
                    _ => log::debug!("unhandled event {event:?}"),
                })
                .menu(&menu)
                .on_menu_event(|app, event| {
                    handle_tray_menu_event(app, &event.id.0);
                })
                .show_menu_on_left_click(true);

            // 统一使用应用默认图标；待托盘模板图标就绪后再启用
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            } else {
                log::warn!("Failed to get default window icon for tray");
            }

            let _tray = tray_builder.build(app)?;
            // 将同一个实例注入到全局状态，避免重复创建导致的不一致
            app.manage(app_state);

            // 初始化 SkillService
            match SkillService::new() {
                Ok(skill_service) => {
                    app.manage(commands::skill::SkillServiceState(Arc::new(skill_service)));
                }
                Err(e) => {
                    log::warn!("初始化 SkillService 失败: {e}");
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_providers,
            commands::get_current_provider,
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::switch_provider,
            commands::import_default_config,
            commands::get_claude_config_status,
            commands::get_config_status,
            commands::get_claude_code_config_path,
            commands::get_config_dir,
            commands::open_config_folder,
            commands::pick_directory,
            commands::open_external,
            commands::get_init_error,
            commands::get_migration_result,
            commands::get_app_config_path,
            commands::open_app_config_folder,
            commands::get_claude_common_config_snippet,
            commands::set_claude_common_config_snippet,
            commands::get_common_config_snippet,
            commands::set_common_config_snippet,
            commands::read_live_provider_settings,
            commands::get_settings,
            commands::save_settings,
            commands::restart_app,
            commands::check_for_updates,
            commands::is_portable_mode,
            commands::get_claude_plugin_status,
            commands::read_claude_plugin_config,
            commands::apply_claude_plugin_config,
            commands::is_claude_plugin_applied,
            // Claude MCP management
            commands::get_claude_mcp_status,
            commands::read_claude_mcp_config,
            commands::upsert_claude_mcp_server,
            commands::delete_claude_mcp_server,
            commands::validate_mcp_command,
            // usage query
            commands::queryProviderUsage,
            commands::testUsageScript,
            // New MCP via config.json (SSOT)
            commands::get_mcp_config,
            commands::upsert_mcp_server_in_config,
            commands::delete_mcp_server_in_config,
            commands::set_mcp_enabled,
            // v3.7.0: Unified MCP management
            commands::get_mcp_servers,
            commands::upsert_mcp_server,
            commands::delete_mcp_server,
            commands::toggle_mcp_app,
            // Prompt management
            commands::get_prompts,
            commands::upsert_prompt,
            commands::delete_prompt,
            commands::enable_prompt,
            commands::import_prompt_from_file,
            commands::get_current_prompt_file_content,
            // ours: endpoint speed test + custom endpoint management
            commands::test_api_endpoints,
            commands::get_custom_endpoints,
            commands::add_custom_endpoint,
            commands::remove_custom_endpoint,
            commands::update_endpoint_last_used,
            // app_config_dir override via Store
            commands::get_app_config_dir_override,
            commands::set_app_config_dir_override,
            // provider sort order management
            commands::update_providers_sort_order,
            // theirs: config import/export and dialogs
            commands::export_config_to_file,
            commands::import_config_from_file,
            commands::save_file_dialog,
            commands::open_file_dialog,
            commands::sync_current_providers_live,
            update_tray_menu,
            // Environment variable management
            commands::check_env_conflicts,
            commands::delete_env_vars,
            commands::restore_env_backup,
            // Skill management
            commands::get_skills,
            commands::install_skill,
            commands::uninstall_skill,
            commands::get_skill_repos,
            commands::add_skill_repo,
            commands::remove_skill_repo,
            // Auto launch
            commands::set_auto_launch,
            commands::get_auto_launch_status,
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        {
            match event {
                // macOS 在 Dock 图标被点击并重新激活应用时会触发 Reopen 事件，这里手动恢复主窗口
                RunEvent::Reopen { .. } => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = window.set_skip_taskbar(false);
                        }
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        apply_tray_policy(app_handle, true);
                    }
                }
                _ => {}
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app_handle, event);
        }
    });
}

// ============================================================
// 迁移错误对话框辅助函数
// ============================================================

/// 检测是否为中文环境
fn is_chinese_locale() -> bool {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .map(|lang| lang.starts_with("zh"))
        .unwrap_or(false)
}

/// 显示迁移错误对话框
/// 返回 true 表示用户选择重试，false 表示用户选择退出
fn show_migration_error_dialog(app: &tauri::AppHandle, error: &str) -> bool {
    let title = if is_chinese_locale() {
        "配置迁移失败"
    } else {
        "Migration Failed"
    };

    let message = if is_chinese_locale() {
        format!(
            "从旧版本迁移配置时发生错误：\n\n{error}\n\n\
            您的数据尚未丢失，旧配置文件仍然保留。\n\
            建议回退到旧版本 CC Switch 以保护数据。\n\n\
            点击「重试」重新尝试迁移\n\
            点击「退出」关闭程序（可回退版本后重新打开）"
        )
    } else {
        format!(
            "An error occurred while migrating configuration:\n\n{error}\n\n\
            Your data is NOT lost - the old config file is still preserved.\n\
            Consider rolling back to an older CC Switch version.\n\n\
            Click 'Retry' to attempt migration again\n\
            Click 'Exit' to close the program"
        )
    };

    let retry_text = if is_chinese_locale() {
        "重试"
    } else {
        "Retry"
    };
    let exit_text = if is_chinese_locale() {
        "退出"
    } else {
        "Exit"
    };

    // 使用 blocking_show 同步等待用户响应
    // OkCancelCustom: 第一个按钮（重试）返回 true，第二个按钮（退出）返回 false
    app.dialog()
        .message(&message)
        .title(title)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(
            retry_text.to_string(),
            exit_text.to_string(),
        ))
        .blocking_show()
}
