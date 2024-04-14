use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use cxsign_internal::{
    store::{tables::ExcludeTable, DataBaseTableTrait},
    Activity, Course, DefaultGestureOrSigncodeSignner, DefaultLocationSignner,
    DefaultNormalOrRawSignner, DefaultPhotoSignner, RawSign, Session, Sign, SignResult, SignTrait,
    SignnerTrait,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

use crate::{
    signner::TauriQrCodeSignner, AccountPair, CurrentSignState, CurrentSignUnamesState,
    DataBaseState, SessionsState,
};
#[derive(Serialize)]
pub struct RawSignPair {
    sign: RawSign,
    unames: Vec<AccountPair>,
}

#[derive(Deserialize)]
pub struct LocationSignnerInfo {
    pub location_str: Option<String>,
    pub no_random_shift: bool,
}
impl FromStr for LocationSignnerInfo {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[tauri::command]
pub async fn list_course_activities(
    course: Course,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<Vec<RawSign>, String> {
    let db = db_state.0.lock().unwrap();
    let table = ExcludeTable::from_ref(&db);
    let sessions = sessions_state.0.lock().unwrap();
    let r = if let Some((_uname, session)) = sessions.iter().next() {
        let (v, _, _) =
            Activity::get_course_activities(table, session, &course).map_err(|e| e.to_string())?;
        v
    } else {
        vec![]
    };
    Ok(r)
}

#[tauri::command]
pub async fn list_all_activities(
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<Vec<RawSignPair>, String> {
    let db = db_state.0.lock().unwrap();
    let table = ExcludeTable::from_ref(&db);
    let sessions = sessions_state.0.lock().unwrap();
    let (r, _, _) =
        Activity::get_all_activities(table, sessions.values(), false).map_err(|e| e.to_string())?;
    Ok(r.into_iter()
        .map(|(sign, sessions)| RawSignPair {
            sign,
            unames: sessions.into_iter().map(AccountPair::from).collect(),
        })
        .collect())
}

#[tauri::command]
pub async fn prepare_sign(
    sign: RawSign,
    accounts: Vec<AccountPair>,
    sessions_state: tauri::State<'_, SessionsState>,
    sign_state: tauri::State<'_, CurrentSignState>,
) -> Result<(), String> {
    let sessions_ = sessions_state.0.lock().unwrap();
    let mut sessions = HashSet::new();
    for account in accounts {
        if let Some(session) = sessions_.get(account.get_uname()) {
            sessions.insert(session.clone());
        }
    }
    if let Some(session) = sessions.iter().next() {
        let sign = sign.to_sign(session);
        *sign_state.sign.lock().unwrap() = Some(sign);
        *sign_state.accounts.lock().unwrap() = sessions;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_sign_type(
    sign_state: tauri::State<'_, CurrentSignState>,
) -> Result<String, String> {
    let t = sign_state
        .sign
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| match s {
            Sign::Photo(_) => "photo",
            Sign::Normal(_) => "normal",
            Sign::QrCode(_) => "qrcode",
            Sign::Gesture(_) => "gesture",
            Sign::Location(_) => "location",
            Sign::Signcode(_) => "signcode",
            Sign::Unknown(_) => "unknown",
        })
        .unwrap_or("unknown")
        .to_string();
    Ok(t)
}
fn handle_results(results: HashMap<&Session, SignResult>, app_handle: &tauri::AppHandle) {
    for (session, result) in results {
        match result {
            SignResult::Susses => {
                info!("签到成功：{}", session.get_stu_name());
                app_handle.emit("sign:susses", session.get_uname()).unwrap();
            }
            SignResult::Fail { msg } => {
                info!("签到失败：{}", session.get_stu_name());
                app_handle
                    .emit("sign:fail", [session.get_uname(), &msg])
                    .unwrap();
            }
        }
    }
}
#[tauri::command]
pub async fn sign_single(
    db_state: tauri::State<'_, DataBaseState>,
    sign_state: tauri::State<'_, CurrentSignState>,
    unames_state: tauri::State<'_, CurrentSignUnamesState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let sign = Arc::clone(&sign_state.sign);
    let sign = sign.lock().unwrap().clone();
    if let Some(sign) = sign {
        let db = Arc::clone(&db_state.0);
        let sessions = Arc::clone(&sign_state.accounts);
        let sign_name = sign.as_inner().name.clone();
        let app_handle_ = app_handle.clone();
        let unames = Arc::clone(&unames_state.0);
        match sign {
            Sign::Photo(sign) => {
                info!("签到[{sign_name}]为拍照签到。");
                app_handle.listen("sign:photo", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let sign = sign.clone();
                    let app = app_handle_.clone();
                    let sessions = Arc::clone(&sessions);
                    let unames = Arc::clone(&unames);
                    app_handle_
                        .dialog()
                        .file()
                        .add_filter("选取图片", &["png", "jpeg"])
                        .pick_file(move |file_response| {
                            let mut sign = sign.clone();
                            let sign = &mut sign;
                            let path = file_response.map(|p| p.path);
                            let unames = unames.lock().unwrap();
                            let sessions = sessions.lock().unwrap();
                            let results = DefaultPhotoSignner::new(&path).sign(
                                sign,
                                sessions.iter().filter(|a| unames.contains(a.get_uname())),
                            );
                            if let Ok(results) = results {
                                handle_results(results, &app)
                            }
                        });
                });
            }
            Sign::Normal(sign) => {
                info!("签到[{sign_name}]为普通签到。");
                app_handle.listen("sign:normal", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let unames = unames.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultNormalOrRawSignner.sign(
                        sign,
                        sessions.iter().filter(|a| unames.contains(a.get_uname())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::QrCode(sign) => {
                info!("签到[{sign_name}]为二维码签到。");
                match sign {
                    cxsign_internal::QrCodeSign::RefreshQrCodeSign(sign) => {
                        let mut sign = sign.clone();
                        let sign = &mut sign;
                        let _ = TauriQrCodeSignner::new(Arc::clone(&db), app_handle_.clone())
                            .sign(sign, None.iter());
                    }
                    cxsign_internal::QrCodeSign::NormalQrCodeSign(sign) => {
                        let mut sign = sign.clone();
                        let sign = &mut sign;
                        let _ = TauriQrCodeSignner::new(Arc::clone(&db), app_handle_.clone())
                            .sign(sign, None.iter());
                    }
                }
            }
            Sign::Gesture(sign) => {
                info!("签到[{sign_name}]为手势签到。");
                app_handle.listen("sign:gesture", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let unames = unames.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) =
                        DefaultGestureOrSigncodeSignner::new(p.payload().trim_matches(|c| c == '"'))
                            .sign(
                                sign,
                                sessions.iter().filter(|a| unames.contains(a.get_uname())),
                            )
                    {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Location(sign) => {
                info!("签到[{sign_name}]为位置签到。");
                // sign_results = DefaultLocationSignner::new(db, 位置字符串, *是否禁用随机偏移)
                //     .sign(sign, sessions)?;
                app_handle.listen("sign:location", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let LocationSignnerInfo {
                        location_str,
                        no_random_shift,
                    } = p.payload().parse().unwrap();
                    let unames = unames.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultLocationSignner::new(
                        &db.lock().unwrap(),
                        &location_str,
                        no_random_shift,
                    )
                    .sign(
                        sign,
                        sessions.iter().filter(|a| unames.contains(a.get_uname())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Signcode(sign) => {
                info!("签到[{sign_name}]为签到码签到。");
                app_handle.listen("sign:signcode", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let unames = unames.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) =
                        DefaultGestureOrSigncodeSignner::new(p.payload().trim_matches(|c| c == '"'))
                            .sign(
                                sign,
                                sessions.iter().filter(|a| unames.contains(a.get_uname())),
                            )
                    {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Unknown(sign) => {
                warn!("签到[{}]为无效签到类型！", sign.name);
                app_handle.listen("sign:unknown", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let unames = unames.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultNormalOrRawSignner.sign(
                        sign,
                        sessions.iter().filter(|a| unames.contains(a.get_uname())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_uname(
    uname: String,
    state: tauri::State<'_, CurrentSignUnamesState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().remove(&uname))
}
#[tauri::command]
pub async fn add_uname(
    uname: String,
    state: tauri::State<'_, CurrentSignUnamesState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().insert(uname))
}
#[tauri::command]
pub async fn add_unames(
    unames: Vec<String>,
    state: tauri::State<'_, CurrentSignUnamesState>,
) -> Result<(), String> {
    info!("添加：{unames:?}");
    state.0.lock().unwrap().extend(unames);
    Ok(())
}

#[tauri::command]
pub async fn has_uname(
    uname: String,
    state: tauri::State<'_, CurrentSignUnamesState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().contains(&uname))
}
#[tauri::command]
pub async fn clear_unames(state: tauri::State<'_, CurrentSignUnamesState>) -> Result<(), String> {
    state.0.lock().unwrap().clear();
    Ok(())
}
