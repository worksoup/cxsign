#![feature(let_chains)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(iter_next_chunk)]
#![feature(iter_array_chunks)]
#![feature(map_try_insert)]

mod command;
mod signner;
mod state;
mod tools;

use cxsign_internal::store::tables::AccountTable;
use cxsign_internal::store::tables::AliasTable;
use cxsign_internal::store::tables::CourseTable;
use cxsign_internal::store::tables::ExcludeTable;
use cxsign_internal::store::tables::LocationTable;
use log::{debug, info, trace};
use state::*;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::Manager;

use command::*;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    #[cfg(mobile)]
    let default_builder = tauri::Builder::default().plugin(tauri_plugin_barcode_scanner::init());
    #[cfg(not(mobile))]
    let default_builder = tauri::Builder::default();
    // #[cfg(target_os = "android")]
    // let default_builder = default_builder.plugin(file_picker_android::init());
    // #[cfg(not(target_os = "android"))]
    let default_builder = default_builder.plugin(tauri_plugin_dialog::init());
    default_builder
        .setup(|app| {
            #[cfg(mobile)]
            let dir = {
                let config_dir = app
                    .path()
                    .resolve("", tauri::path::BaseDirectory::AppLocalData)?;
                cxsign_internal::utils::Dir::from(config_dir)
            };
            #[cfg(not(mobile))]
            let dir = cxsign_internal::utils::DIR.clone();
            app.manage(dir.clone());
            let db = cxsign_internal::store::DataBase::new(dir);
            db.add_table::<AccountTable>();
            db.add_table::<CourseTable>();
            db.add_table::<ExcludeTable>();
            db.add_table::<AliasTable>();
            db.add_table::<LocationTable>();
            app.manage(CoursesState::default());
            app.manage(SessionsState::default());
            app.manage(DataBaseState(Arc::new(Mutex::new(db))));
            app.manage(CurrentSignState::default());
            app.manage(CurrentSignUnamesState::default());
            debug!("程序加载。");
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                debug!("开始监听。");
                app_handle.listen("list-rooms:next-step", move |_| {
                    trace!("list-rooms:next-step: set_prog_state_to_false.");
                    PROG_STATE.fetch_and(false, std::sync::atomic::Ordering::Relaxed);
                    info!("操作已取消，准备下一步。");
                });
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            has_accounts,
            add_account,
            refresh_accounts,
            delete_accounts,
            get_config_dir,
            list_accounts,
            load_accounts,
            load_courses,
            list_courses,
            list_course_activities,
            list_all_activities,
            prepare_sign,
            get_sign_type,
            sign_single,
            scan_image,
            remove_uname,
            add_uname,
            has_uname,
            add_unames,
            clear_unames,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
