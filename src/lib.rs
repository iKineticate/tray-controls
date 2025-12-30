use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use tray_icon::menu::{CheckMenuItem, IconMenuItem, MenuId, MenuItem};

type DefaultMenuId = MenuId;

/// Represents different types of checkable menu items with their associated data
///
/// This enum defines three types of checkable menu items:
///
/// ## Variants
///
/// ### `CheckBox`
/// - Contains: `Rc<CheckMenuItem>` and group identifier `G`
/// - Purpose: A standard checkbox that can be checked/unchecked independently
/// - Grouping: Items with the same `G` value belong to the same logical group
///
/// ### `Radio`
/// - Contains: `Rc<CheckMenuItem>`, optional default `MenuId`, and group identifier `G`
/// - Purpose: A radio button where only one item in the same group can be selected
/// - Default ID:   
///         If `Some`, specifies which menu should be selected when all radios in the group are unchecked.   
///         If `None`, no action is taken when all radios are unchecked.   
/// - Grouping: All radio buttons with the same `G` value form a single selection group
///
/// ### `Separate`
/// - Contains: `Rc<CheckMenuItem>` only
/// - Purpose: A standalone checkbox with no grouping requirements
/// - Use case: For independent toggle options that don't belong to any group
///
/// ## Type Parameters
///
/// - `G`: Group identifier type for organizing related checkable items
///   - Used by both `CheckBox` and `Radio` variants
///   - Must implement `Clone` (for storing in the manager)
///   - Typically use `&'static str` or enum variants for type safety
///
/// ## Example
///
/// ```
/// use std::rc::Rc;
/// use tray_controls::CheckMenuKind;
/// use tray_icon::menu::{CheckMenuItem, MenuId};
///
/// // Create a checkbox belonging to "display_group" group
/// let checkbox = CheckMenuItem::with_id("show_toolbar", "Show Toolbar", true, false, None);
/// let check_kind = CheckMenuKind::CheckBox(Rc::new(checkbox), "display_group");
///
/// // Create a radio button in "theme_group" group with default selection
/// let radio = CheckMenuItem::with_id("light_theme", "Light Theme", true, true, None);
/// let radio_kind = CheckMenuKind::Radio(
///     Rc::new(radio),
///     Some(Rc::new(MenuId::new("light_theme"))),
///     "theme_group"
/// );
///
/// // Create a standalone checkbox
/// let separate = CheckMenuItem::new("Auto-save", true, true, None);
/// let separate_kind: CheckMenuKind<&str> = CheckMenuKind::Separate(Rc::new(separate));
/// ```
#[derive(Clone)]
pub enum CheckMenuKind<G> {
    /// A standard checkbox with group association
    /// 
    /// - First parameter: The checkbox menu item
    /// - Second parameter: Group identifier for logical grouping
    CheckBox(Rc<CheckMenuItem>, G),

    /// A radio button with optional default selection and group association
    /// 
    /// - First parameter: The radio button menu item
    /// - Second parameter: Optional default menu ID to select when no radio is checked.
    ///                    If `Some`, this menu will be selected when all radios in the group are unchecked.
    ///                    If `None`, no menu will be selected when all radios are unchecked.
    /// - Third parameter: Group identifier for exclusive selection
    Radio(
        Rc<CheckMenuItem>,
        Option<Rc<DefaultMenuId>>,
        G,
    ),

    /// A standalone checkbox with no group association
    /// 
    /// - Parameter: The standalone checkbox menu item
    Separate(Rc<CheckMenuItem>),
}

#[derive(Clone)]
pub enum MenuControl<G> {
    MenuItem(MenuItem),
    IconMenu(IconMenuItem),
    CheckMenu(CheckMenuKind<G>),
}

impl<G> MenuControl<G> {
    pub fn id(&self) -> &MenuId {
        match self {
            MenuControl::MenuItem(menu_item) => menu_item.id(),
            MenuControl::IconMenu(icon_menu) => icon_menu.id(),
            MenuControl::CheckMenu(check_menu_kind) => match check_menu_kind {
                CheckMenuKind::CheckBox(check_menu, _)
                | CheckMenuKind::Radio(check_menu, _, _)
                | CheckMenuKind::Separate(check_menu) => check_menu.id(),
            },
        }
    }

    pub fn text(&self) -> String {
        match self {
            MenuControl::MenuItem(menu_item) => menu_item.text(),
            MenuControl::IconMenu(icon_menu) => icon_menu.text(),
            MenuControl::CheckMenu(check_menu_kind) => match check_menu_kind {
                CheckMenuKind::CheckBox(check_menu, _)
                | CheckMenuKind::Radio(check_menu, _, _)
                | CheckMenuKind::Separate(check_menu) => check_menu.text(),
            },
        }
    }

    pub fn as_menu_item(&self) -> Option<&MenuItem> {
        match self {
            MenuControl::MenuItem(menu_item) => Some(menu_item),
            _ => None,
        }
    }

    pub fn as_icon_menu(&self) -> Option<&IconMenuItem> {
        match self {
            MenuControl::IconMenu(icon_menu) => Some(icon_menu),
            _ => None,
        }
    }

    pub fn as_check_menu(&self) -> Option<&CheckMenuItem> {
        if let MenuControl::CheckMenu(check_menu) = self {
            let check_menu = match check_menu {
                CheckMenuKind::CheckBox(check_menu, _)
                | CheckMenuKind::Radio(check_menu, _, _)
                | CheckMenuKind::Separate(check_menu) => check_menu,
            };
            Some(check_menu)
        } else {
            None
        }
    }
}

/// Menu manager that provides centralized menu item management and group state handling
///
/// Core features:
/// 1. **Menu storage**: Unified storage for `MenuItem`, `IconMenuItem`, and `CheckMenuItem`
/// 2. **Group management**: Organizes checkbox and radio button groups, ensuring proper radio button logic
/// 3. **Easy access**: Quick access to menu items and their properties via ID
/// 4. **State synchronization**: Automatically updates other buttons in radio groups when one is selected
///
/// The type parameter `G` represents the group identifier for Radio and CheckBox menu items.
/// Must implement: `Clone + Eq + Hash + PartialEq`
/// Recommended to use enums or string constants for type safety and readability.
///
/// # Example
/// ```
/// use std::rc::Rc;
/// use tray_controls::{CheckMenuKind, MenuControl, MenuManager};
/// use tray_icon::menu::{CheckMenuItem, MenuId};
///
/// let mut manager = MenuManager::<&str>::new();
///
/// // Add a checkbox with group ID "display_group"
/// let checkbox = CheckMenuItem::with_id("show_toolbar", "Show Toolbar", true, true, None);
/// manager.insert(MenuControl::CheckMenu(
///     CheckMenuKind::CheckBox(Rc::new(checkbox), "display_group")
/// ));
///
/// // Add radio buttons with group ID "color_group"
/// let radio = CheckMenuItem::with_id("red", "Red", true, true, None);
/// manager.insert(MenuControl::CheckMenu(
///     CheckMenuKind::Radio(
///         Rc::new(radio),
///         Some(Rc::new(MenuId::new("radio default id"))),
///         "color_group"
///     )
/// ));
///
/// // Handle menu clicks - radio groups are automatically synchronized
/// let click_menu_id = MenuId::new("");
///
/// manager.update(&click_menu_id, |menu| {
///     if let Some(menu) = menu {
///         println!("Clicked menu: {}", menu.text());
///     }
/// });
/// ```
///
/// # Example
/// ```
/// use std::rc::Rc;
/// use tray_controls::{CheckMenuKind, MenuControl, MenuManager};
/// use tray_icon::menu::{CheckMenuItem, MenuId};
///
/// #[derive(Clone, Eq, Hash, PartialEq)]
/// enum MenuGroup {
///     CheckBoxDisplay,
///     RadioColor,
/// }
///
/// let mut manager = MenuManager::<MenuGroup>::new();
///
/// // Add a checkbox with group ID "CheckBoxDisplay"
/// let checkbox = CheckMenuItem::with_id("show_toolbar", "Show Toolbar", true, true, None);
/// manager.insert(MenuControl::CheckMenu(
///     CheckMenuKind::CheckBox(Rc::new(checkbox), MenuGroup::CheckBoxDisplay)
/// ));
///
/// // Add radio buttons with group ID "RadioColor", and set the default radio menu ID
/// let radio = CheckMenuItem::with_id("red", "Red", true, true, None);
/// manager.insert(MenuControl::CheckMenu(
///     CheckMenuKind::Radio(
///         Rc::new(radio),
///         Some(Rc::new(MenuId::new("red"))),
///         MenuGroup::RadioColor
///     )
/// ));
///
/// // Handle menu clicks - radio groups are automatically synchronized
/// let click_menu_id = MenuId::new("");
///
/// manager.update(&click_menu_id, |menu| {
///     if let Some(menu) = menu {
///         println!("Clicked menu: {}", menu.text());
///     }
/// });
/// ```
#[derive(Clone)]
pub struct MenuManager<G>
where
    G: Clone + Eq + Hash + PartialEq,
{
    id_to_menu: HashMap<Rc<MenuId>, MenuControl<G>>,
    grouped_check_items: HashMap<G, HashMap<Rc<MenuId>, Rc<CheckMenuItem>>>,
}

impl<G> Default for MenuManager<G>
where
    G: Clone + Eq + Hash + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<G> MenuManager<G>
where
    G: Clone + Eq + Hash + PartialEq,
{
    pub fn new() -> Self {
        MenuManager {
            id_to_menu: HashMap::new(),
            grouped_check_items: HashMap::new(),
        }
    }

    /// Inserts a menu control from the menu manager.
    pub fn insert(&mut self, menu_control: MenuControl<G>) {
        match &menu_control {
            MenuControl::MenuItem(menu_item) => {
                self.id_to_menu
                    .insert(Rc::new(menu_item.id().clone()), menu_control);
            }
            MenuControl::IconMenu(icon_menu) => {
                self.id_to_menu
                    .insert(Rc::new(icon_menu.id().clone()), menu_control);
            }
            MenuControl::CheckMenu(check_menu_mind) => match check_menu_mind {
                CheckMenuKind::Separate(check_menu) => {
                    self.id_to_menu
                        .insert(Rc::new(check_menu.id().clone()), menu_control);
                }
                CheckMenuKind::Radio(check_menu, _default_menu_id, menu_group) => {
                    let menu_id = Rc::new(check_menu.id().clone());
                    let menu_group = menu_group.clone();
                    let check_menu = check_menu.clone();

                    self.id_to_menu.insert(menu_id.clone(), menu_control);
                    self.grouped_check_items
                        .entry(menu_group)
                        .or_default()
                        .insert(menu_id, check_menu);
                }
                CheckMenuKind::CheckBox(check_menu, menu_group) => {
                    let menu_id = Rc::new(check_menu.id().clone());
                    let menu_group = menu_group.clone();
                    let check_menu = check_menu.clone();

                    self.id_to_menu.insert(menu_id.clone(), menu_control);
                    self.grouped_check_items
                        .entry(menu_group)
                        .or_default()
                        .insert(menu_id, check_menu);
                }
            },
        }
    }

    /// Removes a menu control from the menu manager.
    pub fn remove(&mut self, menu_id: &MenuId) {
        let remove_menu = self.id_to_menu.remove(menu_id);

        if let Some(remove_menu) = remove_menu {
            match &remove_menu {
                MenuControl::MenuItem(_) | MenuControl::IconMenu(_) => {}
                MenuControl::CheckMenu(check_menu_kind) => match check_menu_kind {
                    CheckMenuKind::Separate(_) => {}
                    CheckMenuKind::CheckBox(_, group) | CheckMenuKind::Radio(_, _, group) => {
                        if let Some(map) = self.grouped_check_items.get_mut(group) {
                            map.remove(menu_id);
                        }
                    }
                },
            }
        }
    }

    /// Updates the menu control state based on the provided menu ID, and callback the menu control.
    ///
    /// If the menu control is a radio, it ensures that only one item in the group is checked, and callbakc the cheked menu control.
    pub fn update(&mut self, menu_id: &MenuId, callback: impl Fn(Option<&MenuControl<G>>)) {
        let menu_control = self.id_to_menu.get(menu_id);

        if let Some(menu) = menu_control {
            match menu {
                MenuControl::MenuItem(_) | MenuControl::IconMenu(_) => {}
                MenuControl::CheckMenu(check_menu_kind) => match check_menu_kind {
                    CheckMenuKind::CheckBox(_, _) | CheckMenuKind::Separate(_) => {}
                    CheckMenuKind::Radio(check_menu, default_menu_id, group) => {
                        if let Some(check_menus) = self.get_check_items_from_grouped(group) {
                            let click_menu_state = check_menu.is_checked();

                            let (is_checked_menu_id, is_checked_menu) = if click_menu_state {
                                (check_menu.id(), Some(menu))
                            } else {
                                let Some(default_menu_id) = default_menu_id else {
                                    return callback(menu_control);
                                };

                                let default_menu = self.get_menu_item_from_id(default_menu_id);

                                if let Some(MenuControl::CheckMenu(CheckMenuKind::Radio(
                                    menu,
                                    _,
                                    _,
                                ))) = default_menu
                                {
                                    menu.set_checked(true);
                                    (default_menu_id.as_ref(), default_menu)
                                } else {
                                    return callback(menu_control);
                                }
                            };

                            check_menus
                                .iter()
                                .filter(|(menu_id, _)| menu_id.as_ref().ne(is_checked_menu_id))
                                .for_each(|(_, check_menu)| check_menu.set_checked(false));

                            return callback(is_checked_menu);
                        }
                    }
                },
            }
        }

        callback(menu_control);
    }

    /// Gets a menu control from the menu manager based on the provided menu ID.
    pub fn get_menu_item_from_id(&self, menu_id: &MenuId) -> Option<&MenuControl<G>> {
        self.id_to_menu.get(menu_id)
    }

    /// Gets grouped check menu items from the menu manager based on the provided menu group id.
    pub fn get_check_items_from_grouped(
        &self,
        group_id: &G,
    ) -> Option<&HashMap<Rc<MenuId>, Rc<CheckMenuItem>>> {
        self.grouped_check_items.get(group_id)
    }
}
