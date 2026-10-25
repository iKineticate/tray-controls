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
    menu_registry: MenuRegistry<MenuGroup>,
    tray: Option<TrayIcon>,
}

impl App {
    fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        // 2: create MenuRegistry
        let mut menu_registry: MenuRegistry<MenuGroup> = MenuRegistry::new();
        // 3: create and register menu
        let menu = create_register_menu(&mut menu_registry)?;

        let tray = create_tray(menu)?;

        // 4: add MenuRegistry with Global Management
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
                // 5: handle menu event
                match self.menu_registry.handle_event(event.id()) {
                    Err(err) => {
                        println!("Failed to handle menu event: {err}");
                    }
                    Ok(return_menu_meta) => {
                        let return_menu_group = return_menu_meta.group();
                        let return_menu_kind = return_menu_meta.kind();
                        let return_menu_id = return_menu_kind.id();

                        match return_menu_group {
                            // Normal menu items
                            None => match return_menu_kind {
                                MenuItemKind::MenuItem(_m) => {
                                    // If there are only a few ungrouped normal menus,
                                    // you can directly match menu IDs here.
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
                                }
                            },
                            // Grouped menus
                            Some(group) => {
                                match group {
                                    // Handle radio menus
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
                                        let return_menu =
                                            return_menu_kind.as_check_menuitem().unwrap();

                                        println!(
                                            "Click the radio menu(Language): {:?}\n",
                                            return_menu.text()
                                        );
                                        // TODO: do something
                                    }

                                    // Handle checkbox menus
                                    MenuGroup::CheckBoxChange => {
                                        let return_menu =
                                            return_menu_kind.as_check_menuitem().unwrap();

                                        println!(
                                            "Click the checkbox menu(Change): {:?}\n",
                                            return_menu.text()
                                        );
                                        // TODO: do something
                                    } // If multiple tray icons exist,
                                      // normal menus can also be grouped for management.
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

fn create_register_menu(menu_registry: &mut MenuRegistry<MenuGroup>) -> Result<Menu> {
    let separator_menu_item = PredefinedMenuItem::separator();

    // Normal Menu
    let quit_menu_item = MenuItem::with_id("quit", "Quit", true, None);
    menu_registry.register(quit_menu_item.kind(), None);

    // Color Radio Menu
    let color_sub_menu_item = {
        let colors = ["Red", "Green", "Blue"];

        let menu_items = colors
            .into_iter()
            .map(|text| {
                let menu_id = MenuId::new(text.to_ascii_lowercase());
                let is_default_menu = text == "Red";
                let menu = CheckMenuItem::with_id(menu_id, text, true, is_default_menu, None);

                // register Radio Menu
                menu_registry.register_radio(
                    menu.kind(),
                    MenuGroup::RadioColor,
                    Some(MenuId::new("red")),
                );

                menu
            })
            .collect::<Vec<_>>();

        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| check_menu_item as &dyn IsMenuItem)
            .collect();

        Submenu::with_items("Color", true, &menu_items)?
    };

    // Language Radio Menu
    let language_sub_menu_item = {
        let languages = ["English", "Chinise", "Japanese"];

        let menu_items = languages
            .into_iter()
            .map(|text| {
                let menu_id = MenuId::new(text.to_ascii_lowercase());
                let is_default_menu = text == "English";
                let menu = CheckMenuItem::with_id(menu_id, text, true, is_default_menu, None);

                // register Radio Menu
                menu_registry.register_radio(
                    menu.kind(),
                    MenuGroup::RadioLanguage,
                    Some(MenuId::new("english")),
                );

                menu
            })
            .collect::<Vec<_>>();

        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| check_menu_item as &dyn IsMenuItem)
            .collect();

        Submenu::with_items("Language", true, &menu_items)?
    };

    // Change CheckBox Menu
    let change_sub_menu_item = {
        let change = ["Added", "Removed", "Connected", "Disconnected"];

        let menu_items = change
            .into_iter()
            .map(|text| {
                let menu_id = MenuId::new(text.to_ascii_lowercase());
                let menu = CheckMenuItem::with_id(menu_id, text, true, false, None);

                // register CheckBox Menu
                menu_registry.register(menu.kind(), Some(MenuGroup::CheckBoxChange));

                menu
            })
            .collect::<Vec<_>>();

        let menu_items: Vec<&dyn IsMenuItem> = menu_items
            .iter()
            .map(|check_menu_item| check_menu_item as &dyn IsMenuItem)
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
