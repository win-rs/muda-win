use std::{cell::RefCell, rc::Rc};

use crate::{dpi::Position, util::AddOp, ContextMenu, IsMenuItem, MenuId, MenuItemKind};

/// A root menu that can be added to a Window on Windows and Linux
/// and used as the app global menu on macOS.
#[derive(Clone)]
pub struct Menu {
    id: Rc<MenuId>,
    inner: Rc<RefCell<crate::platform_impl::Menu>>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    /// Creates a new menu.
    pub fn new() -> Self {
        let menu = crate::platform_impl::Menu::new(None);
        Self {
            id: Rc::new(menu.id().clone()),
            inner: Rc::new(RefCell::new(menu)),
        }
    }

    /// Creates a new menu with the specified id.
    pub fn with_id<I: Into<MenuId>>(id: I) -> Self {
        let id = id.into();
        Self {
            id: Rc::new(id.clone()),
            inner: Rc::new(RefCell::new(crate::platform_impl::Menu::new(Some(id)))),
        }
    }

    /// Creates a new menu with given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
    pub fn with_items(items: &[&dyn IsMenuItem]) -> crate::Result<Self> {
        let menu = Self::new();
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Creates a new menu with the specified id and given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
    pub fn with_id_and_items<I: Into<MenuId>>(
        id: I,
        items: &[&dyn IsMenuItem],
    ) -> crate::Result<Self> {
        let menu = Self::with_id(id);
        menu.append_items(items)?;
        Ok(menu)
    }

    /// Returns a unique identifier associated with this menu.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Add a menu item to the end of this menu.
    pub fn append(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().add_menu_item(item, AddOp::Append)
    }

    /// Add menu items to the end of this menu. It calls [`Menu::append`] in a loop internally.
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for item in items {
            self.append(*item)?
        }

        Ok(())
    }

    /// Add a menu item to the beginning of this menu.
    pub fn prepend(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(0))
    }

    /// Add menu items to the beginning of this menu. It calls [`Menu::insert_items`] with position of `0` internally.
    pub fn prepend_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        self.insert_items(items, 0)
    }

    /// Insert a menu item at the specified `postion` in the menu.
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .add_menu_item(item, AddOp::Insert(position))
    }

    /// Insert menu items at the specified `postion` in the menu.
    pub fn insert_items(&self, items: &[&dyn IsMenuItem], position: usize) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.insert(*item, position + i)?
        }

        Ok(())
    }

    /// Remove a menu item from this menu.
    pub fn remove(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().remove(item)
    }

    /// Remove the menu item at the specified position from this menu and returns it.
    pub fn remove_at(&self, position: usize) -> Option<MenuItemKind> {
        let mut items = self.items();
        if items.len() > position {
            let item = items.remove(position);
            let _ = self.remove(item.as_ref());
            Some(item)
        } else {
            None
        }
    }

    /// Returns a list of menu items that has been added to this menu.
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.inner.borrow().items()
    }

    /// Adds this menu to a win32 window.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    ///
    /// ##  Note about accelerators:
    ///
    /// For accelerators to work, the event loop needs to call
    /// [`TranslateAcceleratorW`](windows_sys::Win32::UI::WindowsAndMessaging::TranslateAcceleratorW)
    /// with the [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) returned from [`Menu::haccel`]
    ///
    /// #### Example:
    /// ```no_run
    /// # use muda_win::Menu;
    /// # use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, GetMessageW, TranslateMessage, DispatchMessageW, TranslateAcceleratorW};
    /// let menu = Menu::new();
    /// unsafe {
    ///     let mut msg: MSG = std::mem::zeroed();
    ///     while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) == 1 {
    ///         let translated = TranslateAcceleratorW(msg.hwnd, menu.haccel() as _, &msg as *const _);
    ///         if translated != 1{
    ///             TranslateMessage(&msg);
    ///             DispatchMessageW(&msg);
    ///         }
    ///     }
    /// }
    /// ```
    pub unsafe fn init_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow_mut().init_for_hwnd(hwnd)
    }

    /// Adds this menu to a win32 window using the specified theme.
    ///
    /// See [Menu::init_for_hwnd] for more info.
    ///
    /// Note that the theme only affects the menu bar itself and not submenus or context menu.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn init_for_hwnd_with_theme(
        &self,
        hwnd: isize,
        theme: MenuTheme,
    ) -> crate::Result<()> {
        self.inner
            .borrow_mut()
            .init_for_hwnd_with_theme(hwnd, theme)
    }

    /// Set a theme for the menu bar on this window.
    ///
    /// Note that the theme only affects the menu bar itself and not submenus or context menu.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn set_theme_for_hwnd(&self, hwnd: isize, theme: MenuTheme) -> crate::Result<()> {
        self.inner.borrow().set_theme_for_hwnd(hwnd, theme)
    }

    /// Returns The [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) associated with this menu
    /// It can be used with [`TranslateAcceleratorW`](windows_sys::Win32::UI::WindowsAndMessaging::TranslateAcceleratorW)
    /// in the event loop to enable accelerators
    ///
    /// The returned [`HACCEL`](windows_sys::Win32::UI::WindowsAndMessaging::HACCEL) is valid as long as the [Menu] is.
    pub fn haccel(&self) -> isize {
        self.inner.borrow_mut().haccel()
    }

    /// Removes this menu from a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn remove_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow_mut().remove_for_hwnd(hwnd)
    }

    /// Hides this menu from a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn hide_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow().hide_for_hwnd(hwnd)
    }

    /// Shows this menu on a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn show_for_hwnd(&self, hwnd: isize) -> crate::Result<()> {
        self.inner.borrow().show_for_hwnd(hwnd)
    }

    /// Returns whether this menu visible on a on a win32 window
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    pub unsafe fn is_visible_on_hwnd(&self, hwnd: isize) -> bool {
        self.inner.borrow().is_visible_on_hwnd(hwnd)
    }
}

impl ContextMenu for Menu {
    fn hpopupmenu(&self) -> isize {
        self.inner.borrow().hpopupmenu()
    }

    unsafe fn show_context_menu_for_hwnd(&self, hwnd: isize, position: Option<Position>) -> bool {
        self.inner
            .borrow_mut()
            .show_context_menu_for_hwnd(hwnd, position)
    }

    unsafe fn attach_menu_subclass_for_hwnd(&self, hwnd: isize) {
        self.inner.borrow().attach_menu_subclass_for_hwnd(hwnd)
    }

    unsafe fn detach_menu_subclass_from_hwnd(&self, hwnd: isize) {
        self.inner.borrow().detach_menu_subclass_from_hwnd(hwnd)
    }
}

/// The window menu bar theme
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MenuTheme {
    Dark = 0,
    Light = 1,
    Auto = 2,
}
