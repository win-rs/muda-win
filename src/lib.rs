#![allow(clippy::uninlined_format_args)]

//! muda-win is a Menu Utilities library for Desktop Applications on Windows.
//!
//!
//! # Notes:
//!
//! - Accelerators don't work unless the win32 message loop calls
//!   [`TranslateAcceleratorW`](https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/WindowsAndMessaging/fn.TranslateAcceleratorW.html).
//!   See [`Menu::init_for_hwnd`](https://docs.rs/muda/latest/x86_64-pc-windows-msvc/muda/struct.Menu.html#method.init_for_hwnd) for more details
//!
//! # Example
//!
//! Create the menu and add your items
//!
//! ```no_run
//! # use muda_win::{Menu, Submenu, MenuItem, accelerator::{Code, Modifiers, Accelerator}, PredefinedMenuItem};
//! let menu = Menu::new();
//! let menu_item2 = MenuItem::new("Menu item #2", false, None);
//! let submenu = Submenu::with_items(
//!     "Submenu Outer",
//!     true,
//!     &[
//!         &MenuItem::new(
//!             "Menu item #1",
//!             true,
//!             Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyD)),
//!         ),
//!         &PredefinedMenuItem::separator(),
//!         &menu_item2,
//!         &MenuItem::new("Menu item #3", true, None),
//!         &PredefinedMenuItem::separator(),
//!         &Submenu::with_items(
//!             "Submenu Inner",
//!             true,
//!             &[
//!                 &MenuItem::new("Submenu item #1", true, None),
//!                 &PredefinedMenuItem::separator(),
//!                 &menu_item2,
//!             ],
//!         ).unwrap(),
//!     ],
//! );
//! ```
//!
//! Then add your root menu to a Window on Windows and Linux
//! or use it as your global app menu on macOS
//!
//! ```no_run
//! # let menu = muda_win::Menu::new();
//! # let window_hwnd = 0;
//! // --snip--
//! unsafe { menu.init_for_hwnd(window_hwnd) };
//! ```
//!
//! # Context menus (Popup menus)
//!
//! You can also use a [`Menu`] or a [`Submenu`] show a context menu.
//!
//! ```no_run
//! use muda_win::ContextMenu;
//! # let menu = muda_win::Menu::new();
//! # let window_hwnd = 0;
//! // --snip--
//! let position = muda_win::dpi::PhysicalPosition { x: 100., y: 120. };
//! unsafe { menu.show_context_menu_for_hwnd(window_hwnd, Some(position.into())) };
//! ```
//! # Processing menu events
//!
//! You can use [`MenuEvent::receiver`] to get a reference to the [`MenuEventReceiver`]
//! which you can use to listen to events when a menu item is activated
//! ```no_run
//! # use muda_win::MenuEvent;
//! #
//! # let save_item: muda_win::MenuItem = unsafe { std::mem::zeroed() };
//! if let Ok(event) = MenuEvent::receiver().try_recv() {
//!     match event.id {
//!         id if id == save_item.id() => {
//!             println!("Save menu item activated");
//!         },
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ### Note for [winit] or [tao] users:
//!
//! You should use [`MenuEvent::set_event_handler`] and forward
//! the menu events to the event loop by using [`EventLoopProxy`]
//! so that the event loop is awakened on each menu event.
//!
//! ```no_run
//! # use winit::event_loop::EventLoop;
//! enum UserEvent {
//!   MenuEvent(muda_win::MenuEvent)
//! }
//!
//! let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
//!
//! let proxy = event_loop.create_proxy();
//! muda_win::MenuEvent::set_event_handler(Some(move |event| {
//!     proxy.send_event(UserEvent::MenuEvent(event));
//! }));
//! ```
//!
//! [`EventLoopProxy`]: https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html
//! [winit]: https://docs.rs/winit
//! [tao]: https://docs.rs/tao

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{LazyLock, OnceLock};

pub mod about_metadata;
pub mod accelerator;
mod builders;
mod error;
mod icon;
mod items;
mod menu;
mod menu_id;
mod platform_impl;
mod util;

pub use about_metadata::AboutMetadata;
pub use builders::*;
pub use dpi;
pub use error::*;
pub use icon::{BadIcon, Icon, NativeIcon};
pub use items::*;
pub use menu::*;
pub use menu_id::MenuId;

/// An enumeration of all available menu types, useful to match against
/// the items returned from [`Menu::items`] or [`Submenu::items`]
#[derive(Clone)]
pub enum MenuItemKind {
    MenuItem(MenuItem),
    Submenu(Submenu),
    Predefined(PredefinedMenuItem),
    Check(CheckMenuItem),
    Icon(IconMenuItem),
}

impl MenuItemKind {
    /// Returns a unique identifier associated with this menu item.
    pub fn id(&self) -> &MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.id(),
            MenuItemKind::Submenu(i) => i.id(),
            MenuItemKind::Predefined(i) => i.id(),
            MenuItemKind::Check(i) => i.id(),
            MenuItemKind::Icon(i) => i.id(),
        }
    }

    /// Casts this item to a [`MenuItem`], and returns `None` if it wasn't.
    pub fn as_menuitem(&self) -> Option<&MenuItem> {
        match self {
            MenuItemKind::MenuItem(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`MenuItem`], and panics if it wasn't.
    pub fn as_menuitem_unchecked(&self) -> &MenuItem {
        match self {
            MenuItemKind::MenuItem(i) => i,
            _ => panic!("Not a MenuItem"),
        }
    }

    /// Casts this item to a [`Submenu`], and returns `None` if it wasn't.
    pub fn as_submenu(&self) -> Option<&Submenu> {
        match self {
            MenuItemKind::Submenu(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`Submenu`], and panics if it wasn't.
    pub fn as_submenu_unchecked(&self) -> &Submenu {
        match self {
            MenuItemKind::Submenu(i) => i,
            _ => panic!("Not a Submenu"),
        }
    }

    /// Casts this item to a [`PredefinedMenuItem`], and returns `None` if it wasn't.
    pub fn as_predefined_menuitem(&self) -> Option<&PredefinedMenuItem> {
        match self {
            MenuItemKind::Predefined(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`PredefinedMenuItem`], and panics if it wasn't.
    pub fn as_predefined_menuitem_unchecked(&self) -> &PredefinedMenuItem {
        match self {
            MenuItemKind::Predefined(i) => i,
            _ => panic!("Not a PredefinedMenuItem"),
        }
    }

    /// Casts this item to a [`CheckMenuItem`], and returns `None` if it wasn't.
    pub fn as_check_menuitem(&self) -> Option<&CheckMenuItem> {
        match self {
            MenuItemKind::Check(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`CheckMenuItem`], and panics if it wasn't.
    pub fn as_check_menuitem_unchecked(&self) -> &CheckMenuItem {
        match self {
            MenuItemKind::Check(i) => i,
            _ => panic!("Not a CheckMenuItem"),
        }
    }

    /// Casts this item to a [`IconMenuItem`], and returns `None` if it wasn't.
    pub fn as_icon_menuitem(&self) -> Option<&IconMenuItem> {
        match self {
            MenuItemKind::Icon(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`IconMenuItem`], and panics if it wasn't.
    pub fn as_icon_menuitem_unchecked(&self) -> &IconMenuItem {
        match self {
            MenuItemKind::Icon(i) => i,
            _ => panic!("Not an IconMenuItem"),
        }
    }

    /// Convert this item into its menu ID.
    pub fn into_id(self) -> MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.into_id(),
            MenuItemKind::Submenu(i) => i.into_id(),
            MenuItemKind::Predefined(i) => i.into_id(),
            MenuItemKind::Check(i) => i.into_id(),
            MenuItemKind::Icon(i) => i.into_id(),
        }
    }
}

/// A trait that defines a generic item in a menu, which may be one of [`MenuItemKind`]
pub trait IsMenuItem: sealed::IsMenuItemBase {
    /// Returns a [`MenuItemKind`] associated with this item.
    fn kind(&self) -> MenuItemKind;
    /// Returns a unique identifier associated with this menu item.
    fn id(&self) -> &MenuId;
    /// Convert this menu item into its menu ID.
    fn into_id(self) -> MenuId;
}

mod sealed {
    pub trait IsMenuItemBase {}
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum MenuItemType {
    MenuItem,
    Submenu,
    Predefined,
    Check,
    Icon,
}

impl Default for MenuItemType {
    fn default() -> Self {
        Self::MenuItem
    }
}

/// A helper trait with methods to help creating a context menu.
pub trait ContextMenu {
    /// Get the popup [`HMENU`] for this menu.
    ///
    /// The returned [`HMENU`] is valid as long as the `ContextMenu` is.
    ///
    /// [`HMENU`]: windows_sys::Win32::UI::WindowsAndMessaging::HMENU
    fn hpopupmenu(&self) -> isize;

    /// Shows this menu as a context menu inside a win32 window.
    ///
    /// - `position` is relative to the window top-left corner, if `None`, the cursor position is used.
    ///
    /// Returns `true` if menu tracking ended because an item was selected, and `false` if menu tracking was cancelled for any reason.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    unsafe fn show_context_menu_for_hwnd(
        &self,
        hwnd: isize,
        position: Option<dpi::Position>,
    ) -> bool;

    /// Attach the menu subclass handler to the given hwnd
    /// so you can recieve events from that window using [MenuEvent::receiver]
    ///
    /// This can be used along with [`ContextMenu::hpopupmenu`] when implementing a tray icon menu.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    unsafe fn attach_menu_subclass_for_hwnd(&self, hwnd: isize);

    /// Remove the menu subclass handler from the given hwnd
    ///
    /// The view must be a pointer to a valid `NSView`.
    ///
    /// # Safety
    ///
    /// The `hwnd` must be a valid window HWND.
    unsafe fn detach_menu_subclass_from_hwnd(&self, hwnd: isize);
}

/// Describes a menu event emitted when a menu item is activated
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MenuEvent {
    /// Id of the menu item which triggered this event
    pub id: MenuId,
}

/// A reciever that could be used to listen to menu events.
pub type MenuEventReceiver = Receiver<MenuEvent>;
pub type MenuEventHandler = Box<dyn Fn(MenuEvent) + Send + Sync + 'static>;

static MENU_CHANNEL: LazyLock<(Sender<MenuEvent>, MenuEventReceiver)> = LazyLock::new(unbounded);
static MENU_EVENT_HANDLER: OnceLock<Option<MenuEventHandler>> = OnceLock::new();

impl MenuEvent {
    /// Returns the id of the menu item which triggered this event
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Gets a reference to the event channel's [`MenuEventReceiver`]
    /// which can be used to listen for menu events.
    ///
    /// ## Note
    ///
    /// This will not receive any events if [`MenuEvent::set_event_handler`] has been called with a `Some` value.
    pub fn receiver<'a>() -> &'a MenuEventReceiver {
        &MENU_CHANNEL.1
    }

    /// Set a handler to be called for new events. Useful for implementing custom event sender.
    ///
    /// ## Note
    ///
    /// Calling this function with a `Some` value,
    /// will not send new events to the channel associated with [`MenuEvent::receiver`]
    pub fn set_event_handler<F: Fn(MenuEvent) + Send + Sync + 'static>(
        f: Option<F>,
    ) -> Option<Option<MenuEventHandler>> {
        if let Some(f) = f {
            // Wrap the closure in a Box to store on the heap
            let boxed_handler = Box::new(f);
            let _ = MENU_EVENT_HANDLER.set(Some(boxed_handler));
        } else {
            let _ = MENU_EVENT_HANDLER.set(None);
        }

        // Dereference and return the Box
        match MENU_EVENT_HANDLER.get() {
            Some(Some(handler)) => Some(Some(Box::new(handler))),
            _ => None,
        }
    }

    pub(crate) fn send(event: MenuEvent) {
        if let Some(handler) = MENU_EVENT_HANDLER.get_or_init(|| None) {
            handler(event);
        } else {
            let _ = MENU_CHANNEL.0.send(event);
        }
    }
}
