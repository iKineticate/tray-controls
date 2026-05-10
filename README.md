# tray-controls

[![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://img.shields.io/crates/v/tray-controls)](https://crates.io/crates/tray-controls)

<h3 align="center"> English | <a href='./README.zh-CN.md'>简体中文</a></h3>

An enhanced menu management utility designed for the **tray-icon** crate.  
It provides grouped management for Radio, CheckBox, and other menu item types, making it especially suitable for projects requiring single-selection menus (Radio) and complex tray menu systems.

# Features

## 🎯 Core Advantages

- Menu Management: Easily manage multiple types of menu items

- Group Management: Automatically manages Radio menu groups to ensure correct single-selection behavior

- Convenient Access: Directly access or modify any menu item and its properties by ID

- Multi-Tray Menu Management: Supports grouped registration for menus from different tray icons, making it easier to manage multiple tray menus

## 🔧 Problems This Crate Solves

When using the `tray-icon` crate, menu event handlers only return the target menu ID instead of the actual menu item object. This makes it:

- Difficult to directly access the target menu object

- Inconvenient to modify menu properties (such as text or checked state)

- Hard to synchronize grouped menu states (such as Radio menus)

- Difficult to manage menus when multiple tray icons exist

This crate solves these problems through a unified menu manager.

# Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
tray-control = "0.2.0"
tray-icon = ">=0.20.0"
```

Example using **winit + tray-icon + tray-control**:

* [`examples/winit.rs`](examples/winit.rs)

Steps:

1. Create a generic type `G` that implements `Clone + Copy + Eq + Hash + PartialEq + std::fmt::Debug`.  
   This generic type `G` is used for grouping **Radio menus** or **CheckBox menus**, making it easier to handle logic inside `tray_icon::MenuEvent::set_event_handler`, for example:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MenuGroup {
    RadioColor, // Only one color can be selected
    RadioLanguage, // Only one language can be selected
    CheckBoxA,
    CheckBoxB,
}
```

2. Create a menu registry (`tray_controls::MenuRegistry<G>`)

```rust
use tray_controls::MenuRegistry;

let mut menu_registry = MenuRegistry::<MenuGroup>::new();
```

3. Create tray menu items, convert them into `tray_icon::menu::MenuItemKind`, and register them into the menu registry (`tray_controls::MenuRegistry<G>`).  
   Items can optionally be grouped.

- Standard menu items:

```rust
use tray_icon::menu::MenuItem;

let mut menu_registry = MenuRegistry::<MenuGroup>::new();

let tray_menu = Menu::new();

let quit_menu_item = MenuItem::with_id("quit", "Quit", true, None);

// Register as a normal menu item
menu_registry.register_normal(quit_menu_item.kind());

let icon_menu_item = IconMenuItem::with_id(
    "icon",
    "Icon",
    true,
    Some(tray_icon::menu::Icon),
    None,
);

// Register as a normal menu item
menu_registry.register_normal(icon_menu_item.kind());

tray_menu.append(&quit_menu_item as &dyn IsMenuItem);
tray_menu.append(&icon_menu_item as &dyn IsMenuItem);
```

- Radio menu items:

```rust
use tray_icon::menu::{Menu, MenuId, CheckMenuItem, IsMenuItem, Submenu};
use tray_controls::MenuRegistry;

let tray_menu = Menu::new();

let language_sub_menu_item = {
    let english_menu_id = MenuId::new("english");
    let chinese_menu_id = MenuId::new("chinese");
    let japanese_menu_id = MenuId::new("japanese");

    let english_menu_item =
        CheckMenuItem::with_id(english_menu_id.clone(), "English", true, true, None);

    let chinese_menu_item =
        CheckMenuItem::with_id(chinese_menu_id, "Chinese", true, false, None);

    let japanese_menu_item =
        CheckMenuItem::with_id(japanese_menu_id, "Japanese", true, false, None);

    let menu_items = [english_menu_item, chinese_menu_item, japanese_menu_item];

    let menu_items: Vec<&dyn IsMenuItem> = menu_items
        .iter()
        .map(|check_menu_item| {
            // Register as a radio menu with a default selected item
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

// Append submenu to tray menu
tray_menu.append(&language_sub_menu_item as &dyn IsMenuItem);
```

4. After all menu items are created and registered, add the menu registry into global state management, such as inside a `winit` App:

```rust
struct App {
    menu_registry: MenuRegistry<MenuGroup>,
    // ...
}

let mut menu_registry = MenuRegistry::<MenuGroup>::new();

let tray_menu = create_register_menu(&mut menu_registry)?;

let tray = create_tray(tray_menu);

let mut app = App {
    menu_registry,
    // ...
};

event_loop.run_app(&mut app);

// fn create_registry_menu(menu_registry: &mut MenuRegistry<MenuGroup>)
// fn create_tray(tray_menu: Menu) -> tray_icon::TrayIcon
```

5. Handle menu events, for example in `winit` by setting an event handler for menu events:

```rust
use tray_controls::MenuRegistry;
use tray_icon::menu::MenuItemKind;

struct App {
    menu_registry: MenuRegistry<MenuGroup>,
    // ...
}

UserEvent::MenuEvent(event) => {
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
                    MenuItemKind::MenuItem(_menu_item) => {
                    // If there are only a few ungrouped normal menus,
                    // you can directly match menu IDs here.
                        match return_menu_id.0.as_str() {
                            "quit" => {
                                // TODO: do something
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
                            // TODO: do something
                        }
                        MenuGroup::RadioLanguage => {
                            // TODO: do something
                        }

                        // Handle checkbox menus
                        MenuGroup::CheckBoxA => {
                            // TODO: do something
                        }
                        MenuGroup::CheckBoxB => {
                            // TODO: do something
                        }

                        // If multiple tray icons exist,
                        // normal menus can also be grouped for management.
                    }
                }
            }
        }
    }
}
```
