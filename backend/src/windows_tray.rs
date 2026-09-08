use anyhow::{Context, Result};
use std::{thread, time::Duration};
use tray_icon::{
    Icon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

enum UserEvent {
    Menu(MenuEvent),
    Tray(TrayIconEvent),
}

#[allow(deprecated)]
pub fn run(url: String) -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .context("failed to create the Windows tray event loop")?;

    let open_item = MenuItem::new("打开管理页面", true, None);
    let quit_item = MenuItem::new("退出 Meshway", true, None);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();
    let menu = Menu::new();
    menu.append(&open_item)
        .context("failed to add the tray open menu item")?;
    menu.append(&quit_item)
        .context("failed to add the tray quit menu item")?;

    let icon = Icon::from_rgba(
        include_bytes!("../assets/meshway-tray.rgba").to_vec(),
        32,
        32,
    )
    .context("failed to load the Meshway tray icon")?;
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Meshway")
        .with_icon(icon)
        .build()
        .context("failed to create the Meshway tray icon")?;

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Tray(event));
    }));

    thread::sleep(Duration::from_millis(350));
    open_browser(&url);

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Wait);
            match event {
                Event::UserEvent(UserEvent::Menu(event)) if event.id() == &open_id => {
                    open_browser(&url);
                }
                Event::UserEvent(UserEvent::Menu(event)) if event.id() == &quit_id => {
                    event_loop.exit();
                }
                Event::UserEvent(UserEvent::Tray(TrayIconEvent::DoubleClick { .. })) => {
                    open_browser(&url);
                }
                _ => {}
            }
        })
        .context("Meshway tray event loop failed")?;

    Ok(())
}

fn open_browser(url: &str) {
    let _ = open::that(url);
}
