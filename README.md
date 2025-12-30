# tray-controls

[![Licence](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE) [![Crates.io version](https://img.shields.io/crates/v/tray-controls)](https://crates.io/crates/tray-controls)

<h3 align="center"> English | <a href="./README.zh-CN.md">简体中文</a> </h3>

An enhanced menu management utility designed for the **tray-icon** crate.
It provides structured management for Radio, CheckBox, and other menu item types, and is especially suitable for applications that require **single-selection menus** or **complex system tray menus**.

---

## Features

### 🎯 Core Advantages

* **Unified Menu Management**
  Easily manage multiple menu item types, including:

  * Standard menu items
  * Icon menu items
  * Checkboxes
  * Radio buttons

* **Group Management**
  Automatically manages Radio menu groups to ensure correct single-selection behavior.

* **Direct Access by ID**
  Access and modify any menu item and its properties directly via its ID.

---

## 🔧 Problems This Crate Solves

When using the `tray-icon` crate, menu event handling only provides the **menu ID**, not the actual menu item object. As a result:

* Direct access to the clicked menu item is difficult
* Updating menu properties (e.g. text, checked state) is inconvenient
* Synchronizing grouped menus (such as Radio buttons) requires manual bookkeeping

This crate solves these issues by introducing a **centralized menu manager** that maintains ownership and state of all menu items.

---

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
tray-control = "0.1.0"
tray-icon = "0.21.2"
```

### Example

An example demonstrating usage with **winit + tray-icon + tray-control**:

* [`examples/winit.rs`](examples/winit.rs)

---

## Core Components

### `MenuControl<G>`

Represents different types of menu items:

```rust
pub enum MenuControl<G> {
    MenuItem(tray_icon::MenuItem),      // Standard menu item
    IconMenu(tray_icon::IconMenuItem),  // Menu item with icon
    CheckMenu(CheckMenuKind<G>),        // Checkbox / Radio menu item
}
```

---

### `CheckMenuKind<G>`

Defines the specific behavior of a checkable menu item:

```rust
pub enum CheckMenuKind<G> {
    CheckBox(Rc<CheckMenuItem>, G), 
    // Checkbox menu item with group identifier

    Radio(Rc<CheckMenuItem>, Option<Rc<DefaultMenuId>>, G), 
    // Radio menu item with option default selection and group identifier

    Separate(Rc<CheckMenuItem>), 
    // Independent checkbox menu item (not grouped)
}
```

---

### `MenuManager<G>`

The core manager responsible for menu storage, grouping, and state synchronization:

```rust
pub struct MenuManager<G>
where
    G: Clone + Eq + Hash + PartialEq,
{ /* private fields */ }
```

---

## Example Code

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
        MenuGroup::CheckBoxA,
    ),
));

manager.insert(MenuControl::CheckMenu(
    CheckMenuKind::Radio(
        Rc::new(radio_menu_item),
        Some(MenuId::new("default_radio_id")),
        MenuGroup::RadioA,
    ),
));

// Use together with tray-icon's MenuEvent::set_event_handler
manager.update(&menu_id, |menu| {
    if let Some(menu) = menu {
        println!("Clicked or toggled menu text: {}", menu.text());
    }
});
```

---

## When to Use This Crate

This crate is particularly useful if:

* You rely on `tray-icon` and need structured menu state management
* Your tray menu includes Radio or grouped CheckBox items
* You want to decouple menu logic from low-level event handling
