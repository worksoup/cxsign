use crate::sign_session::sign_session::SignSession;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    rc::Rc,
    sync::Arc,
};
use tokio::sync::Mutex;

slint::include_modules!();

pub async fn gui() {
    let main_window = MainWindow::new().unwrap();
    let (login_tx, mut login_rx) = futures::channel::mpsc::unbounded();
    let login_tx: &mut UnboundedSender<(String, String)> = Box::leak(Box::new(login_tx));
    let (session_tx, mut session_rx) = futures::channel::mpsc::unbounded();
    let session_tx = Box::leak(Box::new(session_tx));
    let (course_tx, mut course_rx) = futures::channel::mpsc::unbounded();
    let course_tx = Box::leak(Box::new(course_tx));
    main_window.on_login(|uname, pwd| {
        let _ = login_tx.unbounded_send((uname.to_string(), pwd.to_string()));
    });
    let sessions = Arc::new(Mutex::new(HashMap::new()));
    let sessions_write_clone = sessions.clone();
    let sessions_read_clone = sessions.clone();
    let courses = Arc::new(Mutex::new(HashSet::new()));
    let courses_ = courses.clone();
    let main_window_weak: Weak<MainWindow> = main_window.as_weak();
    tokio::spawn(async move {
        fn on_login_tips(main_window: &Weak<MainWindow>) {
            println!("正在登录。");
            main_window
                .upgrade_in_event_loop(|main_window| {
                    println!("正在登录，登录页已禁用！");
                    main_window.set_login_widget_enabled(false);
                })
                .unwrap_or_else(|e| {
                    eprintln!("正在登录，但登录页未能禁用: {e}");
                });
        }
        fn on_login_failed_tips(main_window: &Weak<MainWindow>) {
            eprintln!("登录失败！");
            main_window
                .upgrade_in_event_loop(|main_window| {
                    println!("登录失败，恢复登录页启用状态。");
                    main_window.set_login_widget_enabled(true);
                })
                .unwrap_or_else(|e| {
                    eprintln!("登录失败，但登录页未能恢复启用状态: {e}");
                });
        }
        fn on_login_successed_tips(main_window: &Weak<MainWindow>) {
            println!("登录成功！");
            main_window
                .upgrade_in_event_loop(|main_window| {
                    println!("登录成功，已切换到主页面。");
                    main_window.set_page(MainPage::MainPage);
                    main_window.set_login_widget_enabled(true);
                })
                .unwrap_or_else(|e| {
                    println!("登录成功，但切换到主页面错误: {e}");
                });
        }
        fn on_login_reqead(main_window: &Weak<MainWindow>) {
            println!("该账号已登录！");
            main_window
                .upgrade_in_event_loop(|main_window| {
                    println!("该账号已登录，恢复登录页启用状态。");
                    main_window.set_login_widget_enabled(true);
                })
                .unwrap_or_else(|e| {
                    eprintln!("该账号已登录，但登录页未能恢复启用状态: {e}");
                });
        }
        while let Some((uname, pwd)) = login_rx.next().await {
            // 禁用登录按钮。
            on_login_tips(&main_window_weak);
            // 如果已经登录，则控制台提示，并启用登录按钮。
            if sessions_read_clone.lock().await.contains_key(&uname) {
                on_login_reqead(&main_window_weak);
            } else {
                // 如果还未登录，则控制台提示，并进行登录操作。
                if let Ok(sign_session) = SignSession::login(uname.as_str(), pwd.as_str()).await {
                    if sign_session.get_uid().is_empty() {
                        // 登陆失败。
                        on_login_failed_tips(&main_window_weak);
                    } else {
                        // 登陆成功。
                        let _ = session_tx.unbounded_send((uname, sign_session));
                        on_login_successed_tips(&main_window_weak);
                    }
                } else {
                    // 登陆失败。
                    on_login_failed_tips(&main_window_weak);
                }
            }
        }
    });
    tokio::spawn(async move {
        while let Some((uname, s)) = session_rx.next().await {
            let courses = s.get_courses().await.unwrap();
            let _ = course_tx.unbounded_send(courses.clone());
            sessions_write_clone.lock().await.insert(uname, s);
        }
    });
    let main_window_weak: Weak<MainWindow> = main_window.as_weak();
    tokio::spawn(async move {
        while let Some(courses) = course_rx.next().await {
            for c in courses {
                courses_.lock().await.insert(c);
            }
            let courses_ = courses_.clone();
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                let binding = futures::executor::block_on(courses_.lock());
                let courses_ = binding.deref();
                let courses: Vec<UiCourse> = courses_
                    .iter()
                    .map(|c| UiCourse {
                        name: SharedString::from(c.get_name()),
                        teacher: c.get_teacher_name().into(),
                    })
                    .collect();
                let model = Rc::new(VecModel::from(courses));
                let model = ModelRc::from(model.clone());
                main_window.set_courses(model);
            });
        }
    });
    main_window.run().unwrap();
}
