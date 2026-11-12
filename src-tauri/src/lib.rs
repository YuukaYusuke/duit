mod commands;
pub mod config;
pub mod models;
mod validation;
pub use commands::{
    add_expense, add_income, add_student, backup_database, dashboard, deactivate_student,
    export_csv, export_pdf, import_students, list_expenses, list_incomes, list_students,
    preview_student_import, report, update_expense, update_student,
};
use config::Db;
use rusqlite::Connection;
use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("bendahara.sqlite"))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = db_path(&app.handle()).map_err(std::io::Error::other)?;
            let c = Connection::open(path).map_err(std::io::Error::other)?;
            c.pragma_update(None, "foreign_keys", "ON")?;
            c.execute_batch(include_str!("../migrations/001_initial.sql"))?;
            app.manage(Db(Mutex::new(c)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_students,
            add_student,
            update_student,
            deactivate_student,
            list_incomes,
            add_income,
            list_expenses,
            add_expense,
            update_expense,
            dashboard,
            report,
            preview_student_import,
            import_students,
            export_csv,
            export_pdf,
            backup_database
        ])
        .run(tauri::generate_context!())
        .expect("Gagal menjalankan aplikasi");
}
