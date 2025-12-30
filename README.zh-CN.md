# tray-controls

[![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://img.shields.io/crates/v/tray-controls)](https://crates.io/crates/tray-controls)

<h3 align="center"> 简体中文 | <a href='./README.md'>English</a></h3>

一个为 **tray-icon** 库设计的增强菜单管理工具，提供 Radio、CheckBox 等菜单项的分类管理，特别适合需要单选菜单以及复杂托盘菜单的项目。

# 特性

## 🎯 核心优势
- 菜单管理：轻松管理多种类型的菜单项（普通菜单、图标菜单、复选框、单选按钮）

- 分组管理：自动管理 Radio 菜单的分组状态，确保单选逻辑正确

- 便捷访问：通过 ID 直接访问或设置任意菜单项及其属性

## 🔧 解决的问题

``tray-icon`` 库在设置菜单事件处理器时，仅返回目标菜单的 ID，不返回菜单项对象。这使得：

- 难以直接访问目标菜单项

- 无法方便地调整菜单属性（如文本、选中状态）

- 难以管理分组菜单（如 Radio）的状态同步

本库通过统一的菜单管理器解决了这些问题。

# 使用

添加依赖到你的 `Cargo.toml`:

```toml
[dependencies]
tray-control =  "0.1.0"
tray-icon = "0.21.2"
```
示例使用 **winit + tray-icon + tray-control**：

* [`examples/winit.rs`](examples/winit.rs)

# 核心组件

## MenuControl<G>
表示不同类型的菜单项：
```rust
pub enum MenuControl<G> {
    MenuItem(tray_icon::MenuItem),      // 普通菜单项
    IconMenu(tray_icon::IconMenuItem),  // 图标菜单项
    CheckMenu(CheckMenuKind<G>),        // 复选框 / 单选框菜单项
}
```

## CheckMenuKind<G>
表示复选框菜单的具体类型：
```rust
pub enum CheckMenuKind<G> {
    CheckBox(Rc<CheckMenuItem>, G),      // 复选框菜单，分组标识
    Radio(Rc<CheckMenuItem>, Option<Rc<DefaultMenuId>>, G), // 单选框菜单，可选的默认选中项，分组标识
    Separate(Rc<CheckMenuItem>),         // 独立的复选框菜单
}
```


## MenuManager<G>
核心管理器，提供菜单项的存储、分组和状态管理：
```rust
pub struct MenuManager<G>
where
    G: Clone + Eq + Hash + PartialEq,
{ /* private fields */ }
```


```rust
#[derive(Clone, Eq, Hash, PartialEq)]
enum MenuGroup {
    CheckBoxA,
    CheckBoxB,
    RadioA,
    RadioB,
}

let mut manager = MenuManager::<MenuGroup>::new();

manager.insert(MenuControl::CheckMenu(
    CheckMenuKind::CheckBox(
        Rc::new(checkbox_menu_item),
        MenuGroup::CheckBoxA，
    )
))

manager.insert(MenuControl::CheckMenu(
     CheckMenuKind::Radio(
        Rc::new(radio_menu_item),
        Some(MenuId::new("        Some(MenuId::new("default_radio_id")),
")),
        MenuGroup::RadioA,
    )
));

// 配合 tray-icon 的 MenuEvent::set_event_handler 使用
manager.update(&menu_id, |menu| {
     if let Some(menu) = menu {
         println!("点击或勾选的菜单名称: {}", menu.text());
    }
});
```