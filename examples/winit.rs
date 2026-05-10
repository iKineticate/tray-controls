use anyhow::{Result, anyhow};
use tray_controls::MenuRegistry;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{
        CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem, MenuItemKind,
        PredefinedMenuItem, Submenu,
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

// 1: create your menu group (need add derive)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MenuGroup {
    RadioColor,
    RadioLanguage,
    CheckBoxChange,
}

#[derive(Debug)]
enum UserEvent {
    Exit,
    MenuEvent(MenuEvent),
    UpdateIcon(String),
}

struct App {
    event_loop_proxy: EventLoopProxy<UserEvent>,
    // 2: add MenuRegistry with your menu group
    menu_registry: MenuRegistry<MenuGroup>,
    tray: Option<TrayIcon>,
}

impl App {
    fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        // 3: create MenuRegistry
        let mut menu_registry: MenuRegistry<MenuGroup> = MenuRegistry::new();
        // 4: insert menu controls with your menu group
        let menu = create_menu(&mut menu_registry)?;

        let tray = create_tray(menu)?;

        Ok(App {
            event_loop_proxy,
            menu_registry,
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
            _ => {
                // do something
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Exit => {
                event_loop.exit();
            }
            UserEvent::MenuEvent(event) => {
                // 6: handle menu event
                match self.menu_registry.handle_event(event.id()) {
                    Err(err) => {
                        println!("Failed to handle menu event: {err}");
                    }
                    Ok(return_menu_meta) => {
                        let return_menu_group = return_menu_meta.group();
                        let return_menu_kind = return_menu_meta.kind();
                        let return_menu_id = return_menu_kind.id();

                        match return_menu_group {
                            // normal menu
                            None => match return_menu_kind {
                                MenuItemKind::MenuItem(_m) => {
                                    match return_menu_id.0.as_str() {
                                        "quit" => {
                                            let _ =
                                                self.event_loop_proxy.send_event(UserEvent::Exit);
                                        }
                                        _ => {
                                            // TODO: do something
                                        }
                                    }
                                }
                                MenuItemKind::Check(_check_menu_item) => {
                                    // TODO: do something
                                }
                                MenuItemKind::Icon(_icon_menu_item) => {
                                    // TODO: do something
                                }
                                MenuItemKind::Predefined(_predefined_menu_item) => {
                                    // TODO: do something
                                }
                                _ => {
                                    // Submenu not supported
                                } // If the type of the menu in the ungrouped menu is only [MenuItem], you can do something by directly matching the ID of the returned menu
                                  // match return_menu_id.0.as_str(){
                                  //     "quit" => {
                                  //         let _ = self.event_loop_proxy.send_event(UserEvent::Exit);
                                  //     }
                                  //     "about" => {
                                  //         // do something
                                  //     }
                                  //     _ => {
                                  //         // do something
                                  //     }
                                  // }
                            },
                            // menu group
                            Some(group) => {
                                match group {
                                    // 7(1): handle radio menu
                                    MenuGroup::RadioColor => {
                                        // The [as_check_menuitem()] method will not panic because in the handle_event() method,
                                        // it has been determined that if the menu in the [Checkbox] or [Radio] group is not of the [CheckMenuItem] type,
                                        // an error will be returned directly.
                                        let return_menu =
                                            return_menu_kind.as_check_menuitem().unwrap();

                                        println!(
                                            "Check the radio Menu(Color): {:?}\n",
                                            return_menu.text()
                                        );

                                        let _ = self.event_loop_proxy.send_event(
                                            UserEvent::UpdateIcon(return_menu_id.0.clone()),
                                        );
                                    }
                                    MenuGroup::RadioLanguage => {
                                        println!(
                                            "Click the radio menu(Language): {:?}\n",
                                            return_menu_id
                                        );
                                        // TODO: do something
                                    }

                                    // 7(2): handle checkbox menu
                                    MenuGroup::CheckBoxChange => {
                                        println!("Click the checkbox menu: {:?}\n", return_menu_id);
                                        // TODO: do something
                                    } // handle other non-checkmenuitem menu groups (for multiple TrayIcon)
                                }
                            }
                        }
                    }
                }
            }
            UserEvent::UpdateIcon(color) => {
                let color = if color == "red" {
                    [255u8, 0, 0, 255]
                } else if color == "green" {
                    [0u8, 255, 0, 255]
                } else if color == "blue" {
                    [0u8, 0, 255, 255]
                } else {
                    return;
                };

                let icon = create_icon(color);

                if let Some(tray) = &self.tray {
                    let _ = tray.set_icon(Some(icon));
                }
            }
        }
    }
}

// 5: create menu and insert menu controls with your menu group
fn create_menu(menu_registry: &mut MenuRegistry<MenuGroup>) -> Result<Menu> {
    let separator_menu_item = PredefinedMenuItem::separator();

    // Normal Menu
    let quit_menu_item = MenuItem::with_id("quit", "Quit", true, None);
    menu_registry.register_normal(quit_menu_item.kind());

    // Color Radio Menu
    let color_sub_menu_item = {
        let red_menu_id = MenuId::new("red");
        let green_menu_id = MenuId::new("green");
        let blue_menu_id = MenuId::new("blue");

        let red_menu_item = CheckMenuItem::with_id(red_menu_id.clone(), "Red", true, true, None);
        let green_menu_item =
            CheckMenuItem::with_id(green_menu_id.clone(), "Green", true, false, None);
        let blue_menu_item =
            CheckMenuItem::with_id(blue_menu_id.clone(), "Blue", true, false, None);

        let menu_items = [red_menu_item, green_menu_item, blue_menu_item];
        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| {
                // register Radio Menu
                menu_registry.register_radio(
                    check_menu_item.kind(),
                    MenuGroup::RadioColor,
                    Some(red_menu_id.clone()),
                );

                check_menu_item as &dyn IsMenuItem
            })
            .collect();

        Submenu::with_items("Color", true, &menu_items)?
    };

    // Language Radio Menu
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
                // register Radio Menu
                menu_registry.register_radio(
                    check_menu_item.kind(),
                    MenuGroup::RadioLanguage,
                    Some(english_menu_id.clone()),
                );

                check_menu_item as &dyn IsMenuItem
            })
            .collect();

        Submenu::with_items("Language", true, &menu_items)?
    };

    // Change CheckBox Menu
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
                // register CheckBox Menu
                menu_registry.register_checkbox(check_menu_item.kind(), MenuGroup::CheckBoxChange);

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
