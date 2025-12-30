use std::rc::Rc;

use anyhow::{Result, anyhow};
use tray_controls::{CheckMenuKind, MenuControl, MenuManager};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{
        CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    },
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

fn main() -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(UserEvent::MenuEvent(event))
            .expect("Failed to send MenuEvent");
    }));

    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy)?;
    event_loop.run_app(&mut app)?;

    Ok(())
}

// need add derive
#[derive(Clone, PartialEq, Eq, Hash)]
enum MenuGroup {
    RadioColor,
    RadioLanguage,
    CheckBoxChange,
}

#[derive(Debug)]
enum UserEvent {
    Exit,
    MenuEvent(MenuEvent),
}

struct App {
    event_loop_proxy: EventLoopProxy<UserEvent>,
    menu_manager: MenuManager<MenuGroup>,
    tray: Option<TrayIcon>,
}

impl App {
    fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        let mut menu_manager: MenuManager<MenuGroup> = MenuManager::new();
        let menu = create_menu(&mut menu_manager)?;
        let tray = create_tray(menu)?;

        Ok(App {
            event_loop_proxy,
            menu_manager,
            tray: Some(tray),
        })
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Exit => {
                event_loop.exit();
            }
            UserEvent::MenuEvent(event) => {
                let click_menu_id = event.id();
                self.menu_manager.update(click_menu_id, |menu_control| {
                    if let Some(menu_control) = menu_control {
                        match menu_control {
                            MenuControl::CheckMenu(check_menu_kind) => match check_menu_kind {
                                CheckMenuKind::Separate(check_menu) => {
                                    println!(
                                        "Click the Separate Check Menu: {:?}\n",
                                        check_menu.text()
                                    );
                                }
                                CheckMenuKind::CheckBox(check_menu, group) => {
                                    match group {
                                        MenuGroup::CheckBoxChange => {
                                            println!(
                                                "Click the Check Box Menu(Change): {:?}\n",
                                                check_menu.text()
                                            );
                                            // TODO: do something
                                        }
                                        // your check box menu group id
                                        _ => {}
                                    }
                                }
                                CheckMenuKind::Radio(checked_menu, default_menu_id, group) => {
                                    if default_menu_id.as_ref() == checked_menu.id() {
                                        println!("The Radio Menu is check default")
                                    }

                                    match group {
                                        MenuGroup::RadioColor => {
                                            println!(
                                                "Check the Radio Menu(Color): {:?}\n",
                                                checked_menu.text()
                                            );
                                            let color = if checked_menu.id().0 == "red" {
                                                [255u8, 0, 0, 255]
                                            } else if checked_menu.id().0 == "green" {
                                                [0u8, 255, 0, 255]
                                            } else if checked_menu.id().0 == "blue" {
                                                [0u8, 0, 255, 255]
                                            } else {
                                                return;
                                            };

                                            let icon = create_icon(color);

                                            if let Some(tray) = &self.tray {
                                                let _ = tray.set_icon(Some(icon));
                                            }
                                        }
                                        MenuGroup::RadioLanguage => {
                                            println!(
                                                "Check the Radio Menu(Language): {:?}\n",
                                                checked_menu.text()
                                            );
                                            // TODO: change language
                                        }
                                        _ => {}
                                    }
                                }
                            },
                            MenuControl::IconMenu(icon_menu) => {
                                println!("Click Icon Menu: {:?}\n", icon_menu.text());
                                // TODO: do something
                            }
                            MenuControl::MenuItem(menu_item) => {
                                println!("Click Menu Item: {:?}\n", menu_item.text());
                                if click_menu_id.0 == "quit" {
                                    let _ = self.event_loop_proxy.send_event(UserEvent::Exit);
                                }
                                // else if click_menu_id.0 == "your menu id" {
                                //     // TODO: do something
                                // }
                            }
                        }
                    }
                });
            }
        }
    }
}

fn create_menu(menu_manager: &mut MenuManager<MenuGroup>) -> Result<Menu> {
    let separator_menu_item = PredefinedMenuItem::separator();

    let quit_menu_id = MenuId::new("quit");
    let quit_menu_item = MenuItem::with_id(quit_menu_id, "Quit", true, None);
    menu_manager.insert(MenuControl::MenuItem(quit_menu_item.clone()));

    // Color Radio Check Menu
    let color_sub_menu_item = {
        let red_menu_id = MenuId::new("red");
        let green_menu_id = MenuId::new("green");
        let blue_menu_id = MenuId::new("blue");

        let red_menu_item = CheckMenuItem::with_id(red_menu_id.clone(), "Red", true, true, None);
        let green_menu_item = CheckMenuItem::with_id(green_menu_id, "Green", true, false, None);
        let blue_menu_item = CheckMenuItem::with_id(blue_menu_id, "Blue", true, false, None);

        let menu_items = [red_menu_item, green_menu_item, blue_menu_item];
        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| {
                menu_manager.insert(MenuControl::CheckMenu(CheckMenuKind::Radio(
                    Rc::new(check_menu_item.clone()),
                    Rc::new(red_menu_id.clone()),
                    MenuGroup::RadioColor,
                )));

                check_menu_item as &dyn IsMenuItem
            })
            .collect();

        Submenu::with_items("Color", true, &menu_items)?
    };

    // Language Radio Check Menu
    let language_sub_menu_item = {
        let english_menu_id = MenuId::new("english");
        let chinise_menu_id = MenuId::new("chinise");
        let japanese_menu_id = MenuId::new("japanese");

        let english_menu_item =
            CheckMenuItem::with_id(english_menu_id.clone(), "English", true, true, None);
        let chinise_menu_item =
            CheckMenuItem::with_id(chinise_menu_id, "Chinise", true, false, None);
        let japanese_menu_item =
            CheckMenuItem::with_id(japanese_menu_id, "Japanese", true, false, None);

        let menu_items = [english_menu_item, chinise_menu_item, japanese_menu_item];
        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| {
                menu_manager.insert(MenuControl::CheckMenu(CheckMenuKind::Radio(
                    Rc::new(check_menu_item.clone()),
                    Rc::new(english_menu_id.clone()),
                    MenuGroup::RadioLanguage,
                )));

                check_menu_item as &dyn IsMenuItem
            })
            .collect();

        Submenu::with_items("Language", true, &menu_items)?
    };

    // CheckBoxChange Check Box Menu
    let change_sub_menu_item = {
        let added_menu_id = MenuId::new("added");
        let removed_menu_id = MenuId::new("removed");
        let connected_menu_id = MenuId::new("connected");
        let disconnected_menu_id = MenuId::new("disconnected");

        let added_menu_item = CheckMenuItem::with_id(added_menu_id, "Added", true, false, None);
        let removed_menu_item =
            CheckMenuItem::with_id(removed_menu_id, "Removed", true, false, None);
        let connected_menu_item =
            CheckMenuItem::with_id(connected_menu_id, "Connected", true, false, None);
        let disconnected_menu_item =
            CheckMenuItem::with_id(disconnected_menu_id, "Disconnected", true, false, None);

        let menu_items = [
            added_menu_item,
            removed_menu_item,
            connected_menu_item,
            disconnected_menu_item,
        ];
        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| {
                menu_manager.insert(MenuControl::CheckMenu(CheckMenuKind::CheckBox(
                    Rc::new(check_menu_item.clone()),
                    MenuGroup::CheckBoxChange,
                )));

                check_menu_item as &dyn IsMenuItem
            })
            .collect();

        Submenu::with_items("Change", true, &menu_items)?
    };

    Menu::with_items(&[
        &color_sub_menu_item as &dyn IsMenuItem,
        &separator_menu_item as &dyn IsMenuItem,
        &language_sub_menu_item as &dyn IsMenuItem,
        &separator_menu_item as &dyn IsMenuItem,
        &change_sub_menu_item as &dyn IsMenuItem,
        &separator_menu_item as &dyn IsMenuItem,
        &quit_menu_item as &dyn IsMenuItem,
    ])
    .map_err(|e| anyhow!("failed to crate tray menu: {e}"))
}

fn create_tray(menu: Menu) -> Result<TrayIcon> {
    let dafault_red_color = [255u8, 0, 0, 255];

    TrayIconBuilder::new()
        .with_menu_on_left_click(true)
        .with_icon(create_icon(dafault_red_color))
        .with_tooltip("tray-controls")
        .with_menu(Box::new(menu))
        .build()
        .map_err(|e| anyhow!("Failed to build tray - {e}"))
}

fn create_icon(pixiel: [u8; 4]) -> tray_icon::Icon {
    let (width, height) = (16_u32, 16_u32);
    let pixel_count = (width * height) as usize;

    let mut image_data = Vec::with_capacity(pixel_count * 4);

    for _ in 0..pixel_count {
        image_data.extend_from_slice(&pixiel);
    }

    tray_icon::Icon::from_rgba(image_data, width, height).expect("Failed to create icon")
}
