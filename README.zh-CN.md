# tray-controls

[![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://img.shields.io/crates/v/tray-controls)](https://crates.io/crates/tray-controls)

<h3 align="center"> 简体中文 | <a href='./README.md'>English</a></h3>

一个为 **tray-icon** 库设计的增强菜单管理工具，提供 Radio、CheckBox 等菜单项的分组管理，特别适合需要单选菜单（Radio）以及复杂托盘菜单的项目。

# 特性

## 🎯 核心优势
- 菜单管理：轻松管理多种类型的菜单项

- 分组管理：自动管理 Radio 菜单的分组状态，确保单选逻辑正确

- 便捷访问：通过 ID 直接访问或设置任意菜单项及其属性

- 多托盘菜单管理：通过给不同托盘里的菜单进行分组注册，以方便管理多个托盘菜单

## 🔧 解决的问题

``tray-icon`` 库在设置菜单事件处理器时，仅返回目标菜单的 ID，不返回菜单项对象。这使得：

- 难以直接访问目标菜单对象

- 无法方便地调整菜单属性（如文本、选中状态）

- 难以管理分组菜单（如 Radio）的状态同步

- 存在多托盘时难以管理各个托盘的菜单

本库通过统一的菜单管理器解决了这些问题。

# 使用

添加依赖到你的 `Cargo.toml`:

```toml
[dependencies]
tray-control = "0.2.3"
tray-icon = ">=0.20.0"
# 推荐使用 strum 库方便枚举菜单ID
```
示例使用 **winit + tray-icon + tray-control**：

* [`examples/winit.rs`](examples/winit.rs)

步骤：

1. 创建一个要求实现  ``Clone + Copy, Eq + Hash + PartialEq + std::fmt::Debug`` 的 **泛型 G**，该 **泛型 G** 用于 **单选菜单(Radio)** 或 **复选菜单(CheckBox)** 的分组管理，方便在菜单事件回调中 ``tray_icon::MenuEvent::set_event_handler`` 实现对应的功能，例如

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MenuGroup {
    RadioColor, // 颜色单选菜单分组中，仅可选一个颜色
    RadioLanguage, // 语言单选菜单分组中，仅可选一个语言
    CheckBoxA,
    CheckBoxB,
}
``` 

2. 注册一个菜单管理（``tray_controls::MenuRegistry<G>`` ）
```rust
use tray_controls::MenuRegistry;

let mut menu_registry = MenuRegistry::<MenuGroup>::new();
```

3. 创建托盘菜单项目，然后将菜单对象转换为 `tray_icon::menu::MenuItemKind` 并注册到 **菜单注册**（``tray_controls::MenuRegistry<G>``） 中，需要时可分组，例如：

- 普通菜单：

```rust
use tray_icon::menu::MenuItem;

let mut menu_registry = MenuRegistry::<MenuGroup>::new();

let tray_menu = Menu::new();

let quit_menu_item = MenuItem::with_id("quit", "Quit", true, None);
// 将菜单对象转化为 [MenuItemKind] 后注册为普通项目
menu_registry.register(quit_menu_item.kind(), /* 菜单不分组 */ None);

let icon_menu_item = IconMenuItem::with_id("icon", "Icon", true, Some(tray_icon::menu::Icon), None);
// 将菜单对象转化为 [MenuItemKind] 后注册为普通项目
menu_registry.register(icon_menu_item.kind(), /* 菜单不分组 */ None);

tray_menu.append(&quit_menu_item as &dyn IsMenuItem)
tray_menu.append(&icon_menu_item as &dyn IsMenuItem)
```

- 单选菜单：   


```rust
use tray_icon::menu::{MenuId, CheckMenuItem, IsMenuItem, Submenu};
use tray_controls::MenuRegistry;

let language_sub_menu_item = {
    let languages = ["English", "Chinise", "Japanese"];

    let menu_items = languages
        .into_iter()
        .map(|text| {
            let menu_id = MenuId::new(text.to_ascii_lowercase());
            let is_default_menu = text == "English";
            let menu = CheckMenuItem::with_id(menu_id, text, true, is_default_menu, None);

            // 注册为单选项的分组菜单
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

    Submenu::with_items("Language", true, &menu_items).unwrap()
};

// 添加菜单项目至托盘菜单中
let tray_menu = tray_icon::menu::Menu::new();
tray_menu.append(&language_sub_menu_item as &dyn IsMenuItem)
```

4. 当所有菜单项目创建及注册后，将 **菜单注册** 添加至全局管理，如 winit 中的 App { } 中
```rust
struct App {
    menu_registry: MenuRegistry<MenuGroup>,
    // ...
}

let mut menu_registry = MenuRegistry::<MenuGroup>::new();

// 按照上面办法创建菜单并注册菜单
// let tray_menu = ...;
// let tray = ...;

let mut app = App {
    menu_registry,
    // ...
};

event_loop.run_app(&mut app);
```

5. 菜单事件处理，例如在 `winit` 中将为菜单新事件设置要调用的处理程序
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
            println!("failed to handle menu event: {err}");
        }
        Ok(return_menu_meta) => {
            let return_menu_group = return_menu_meta.group();
            let return_menu_kind = return_menu_meta.kind();
            let return_menu_id = return_menu_kind.id();

            match return_menu_group {
                // 无分组的菜单
                None => match return_menu_kind {
                    MenuItemKind::MenuItem(_menu_item) => {
                        // 如果无分类的普通菜单比较少，可以直接匹配菜单ID来处理事件，不需匹配返回菜单的类型
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
                // 存在菜单分组时
                Some(group) => {
                    match group {
                        // 处理 单选菜单
                        MenuGroup::RadioColor => {
                            // TODO: do something
                        }
                        MenuGroup::RadioLanguage => {
                            // TODO: do something
                        }

                        // 处理 多选菜单
                        MenuGroup::CheckBoxA => {
                            // TODO: do something
                        }
                        MenuGroup::CheckBoxB => {
                            // TODO: do something
                        }

                        // 如创建多个托盘，可以给不同托盘的普通菜单分组实现多托盘菜单管理
                    }
                }
            }
        }
    }
}

```