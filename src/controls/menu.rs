/*!
    Menu control definition that integrates with the built-in window type
*/

use std::hash::Hash;
use std::any::TypeId;
use std::mem;
use std::ptr;

use winapi::{HWND, HMENU, UINT, BOOL};

use ui::Ui;
use controls::{Control, ControlT, ControlType, AnyHandle};
use error::Error;

static mut MENU_ITEMS_ID: UINT = 0; 

/**
    A template to create menu controls

    Control specific events: None  

    Members:  
      • `text` : The menu text  
      • `parent` : ID of the parent control to add the menu to  
*/
#[derive(Clone)]
pub struct MenuT<S: Clone+Into<String>, ID: Hash+Clone> {
    pub text: S,
    pub parent: ID,
    pub disabled: bool
}

impl<S: Clone+Into<String>, ID: Hash+Clone> ControlT<ID> for MenuT<S, ID> {
    fn type_id(&self) -> TypeId { TypeId::of::<Menu>() }

    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        let handle_result = unsafe { build_menu(ui, self) };
        match handle_result {
            Ok((h, parent)) => { Ok( Box::new(Menu{handle: h, parent: parent}) as Box<Control> ) },
            Err(e) => Err(e)
        }
    }
}

/**
    A menu control
*/
pub struct Menu {
    handle: HMENU,
    parent: AnyHandle
}

impl Menu {

    /// Return true if the menu is enabled or false otherwise
    pub fn get_enabled(&self) -> bool {
        unsafe{ ::low::menu_helper::is_menu_enabled(self.handle, &self.parent) }
    }

    /// Enable or disable the menu
    pub fn set_enabled(&self, enabled: bool) {
        unsafe{ ::low::menu_helper::enable_menu(self.handle, &self.parent, enabled); }
    }
}

impl Control for Menu {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HMENU(self.handle)
    }

    fn control_type(&self) -> ControlType {
        ControlType::Menu
    }

    fn children(&self) -> Vec<AnyHandle> {
        use low::menu_helper::list_menu_children;
        unsafe{ list_menu_children(self.handle) }
    }

    fn free(&mut self) {
        use user32::DestroyMenu;
        use low::menu_helper::remove_menu_from_parent;

        unsafe{ remove_menu_from_parent(self.handle, &self.parent) };

        unsafe{ DestroyMenu(self.handle) };
    }

}

/**
    A template to create context menu for controls. The menu that appears when the user right click on a control.

    Control specific events: None  
    Members: None
*/
#[derive(Clone)]
pub struct ContextMenuT;

impl<ID: Hash+Clone> ControlT<ID> for ContextMenuT {
    fn type_id(&self) -> TypeId { TypeId::of::<ContextMenu>() }

    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        let handle_result = unsafe { build_context_menu(ui) };
        match handle_result {
            Ok((h, parent)) => {
                 Ok( Box::new(ContextMenu{handle: h, parent: parent}) as Box<Control> ) 
            },
            Err(e) => Err(e)
        }
    }
}


/**
    A context menu control
*/
pub struct ContextMenu {
    handle: HMENU,
    parent: HWND,
}

impl ContextMenu {

    /**
        Show the menu at the selected position.

        Params:
        * `x`: The absolute x position in the screen where the context menu should pop up
        * `y`: The absolute x position in the screen where the context menu should pop up
    */
    pub fn pop_at(&self, x: i32, y: i32) {
        use low::defs::{TrackPopupMenuEx, TPM_NOANIMATION, TPM_RIGHTBUTTON, TPM_LEFTALIGN, TPM_TOPALIGN};
        let flags = TPM_NOANIMATION | TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_TOPALIGN;
        unsafe{ TrackPopupMenuEx(self.handle, flags, x, y, self.parent, ptr::null_mut()); }
    }

}

impl Control for ContextMenu {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HMENU(self.handle)
    }

    fn control_type(&self) -> ControlType {
        ControlType::Menu
    }

    fn children(&self) -> Vec<AnyHandle> {
        use low::menu_helper::list_menu_children;
        unsafe{ list_menu_children(self.handle) }
    }

    fn free(&mut self) {
        use user32::DestroyMenu;
        unsafe{ DestroyMenu(self.handle) };
    }

}


/**
    A template to create menuitems

    Control specific events:  
    `menu::Triggered`

    Members:  
      • `text` : The menu text  
      • `parent` : ID of the parent control to add the menu to  
*/
#[derive(Clone)]
pub struct MenuItemT<S: Clone+Into<String>, ID: Hash+Clone> {
    pub text: S,
    pub parent: ID,
    pub disabled: bool
}

impl<S: Clone+Into<String>, ID: Hash+Clone> ControlT<ID> for MenuItemT<S, ID> {
    fn type_id(&self) -> TypeId { TypeId::of::<MenuItem>() }

   #[allow(unused_variables)]
    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        let handle_result = unsafe { build_menu_item(ui, self) };
        match handle_result {
            Ok((parent, uid)) => { Ok( Box::new(MenuItem{parent: parent, unique_id: uid}) as Box<Control> ) },
            Err(e) => Err(e)
        }
    }
}

/**
    A menu item control
*/ 
pub struct MenuItem {
    parent: HMENU,
    unique_id: UINT
}

impl MenuItem {

    /// Return true if the menuitem is enabled or false otherwise
    pub fn get_enabled(&self) -> bool {
        unsafe{ ::low::menu_helper::is_menuitem_enabled(self.parent, None, Some(self.unique_id)) }
    }

    /// Enable or disable the menuitem
    pub fn set_enabled(&self, enabled: bool) {
        unsafe{ ::low::menu_helper::enable_menuitem(self.parent, None, Some(self.unique_id), enabled); }
    }

}


/**
    A menu item control
*/
impl Control for MenuItem {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HMENU_ITEM(self.parent, self.unique_id)
    }

    fn control_type(&self) -> ControlType { 
        ControlType::MenuItem 
    }

    fn free(&mut self) {
        use low::menu_helper::remove_menu_item_from_parent;
        unsafe{ remove_menu_item_from_parent(self.parent, self.unique_id) };
    }
}


/**
    A menu item separator.

    Control specific events: None  

    Member:  
    * parent: The parent. Must point to a `Menu` control
*/
#[derive(Clone)]
pub struct SeparatorT<ID: Hash+Clone> {
    pub parent: ID
}

impl<ID: Hash+Clone> ControlT<ID> for SeparatorT<ID> {
    fn type_id(&self) -> TypeId { TypeId::of::<Separator>() }

   #[allow(unused_variables)]
    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        let handle_result = unsafe { build_separator(ui, self) };
        match handle_result {
            Ok((parent, uid)) => { Ok( Box::new(Separator{parent: parent, unique_id: uid}) as Box<Control> ) },
            Err(e) => Err(e)
        }
    }
}

/**
    A separator control.
*/
pub struct Separator {
    parent: HMENU,
    unique_id: UINT
}


impl Control for Separator {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HMENU_ITEM(self.parent, self.unique_id)
    }

    fn control_type(&self) -> ControlType { 
        ControlType::MenuItem 
    }

    fn free(&mut self) {
        use low::menu_helper::remove_menu_item_from_parent;
        unsafe{ remove_menu_item_from_parent(self.parent, self.unique_id) };
    }
}


/*
    Private unsafe menu/menuitem methods
*/

#[inline(always)]
unsafe fn build_menu<S: Clone+Into<String>, ID: Clone+Hash>(ui: &Ui<ID>, t: &MenuT<S, ID>) -> Result<(HMENU, AnyHandle), Error> {
    use user32::{CreateMenu, CreatePopupMenu, AppendMenuW, GetMenu, SetMenu, DrawMenuBar};
    use winapi::{MF_STRING, MF_POPUP};
    use low::menu_helper::{use_menu_command, enable_menuitem};
    use low::other_helper::to_utf16;

    let parent_handle = ui.handle_of(&t.parent);
    match &parent_handle {
        &Ok(AnyHandle::HWND(_)) | &Ok(AnyHandle::HMENU(_)) => { /* OK */ },
        &Ok(ref h) => { return Err(Error::BadParent(format!("TreeView or TreeViewItem parent required got \"{}\"", h.human_name()))); },
        &Err(ref e) => { return Err(e.clone()); }
    }

    match parent_handle.unwrap() {
        AnyHandle::HWND(parent_h) => {
            let mut menubar = GetMenu(parent_h);
            if menubar.is_null() {
                // If the window do not have a menu bar, create one
                menubar = CreateMenu();
                use_menu_command(menubar);
                SetMenu(parent_h, menubar);
            }

            let h = CreateMenu();
            use_menu_command(h);

            let text = to_utf16(t.text.clone().into().as_ref());
            AppendMenuW(menubar, MF_STRING|MF_POPUP, mem::transmute(h), text.as_ptr());
            enable_menuitem(menubar, None, None, !t.disabled);
            DrawMenuBar(parent_h); // Draw the menu bar to make sure the changes are visible

            Ok( ( h, AnyHandle::HWND(parent_h)) )
        },
        AnyHandle::HMENU(parent_h) => {
            let h = CreatePopupMenu();
            use_menu_command(h);

            let text = to_utf16(t.text.clone().into().as_ref());
            AppendMenuW(parent_h, MF_STRING|MF_POPUP, mem::transmute(h), text.as_ptr());
            enable_menuitem(parent_h, None, None, !t.disabled);

            Ok( ( h, AnyHandle::HMENU(parent_h) ) )

        },
        _ => unreachable!()
   }
}

#[inline(always)]
unsafe fn build_context_menu<ID: Clone+Hash>(ui: &Ui<ID>) -> Result<(HMENU, HWND), Error> {
    use user32::CreatePopupMenu;
    use low::menu_helper::use_menu_command;

    let menu = CreatePopupMenu();
    use_menu_command(menu);

    Ok((menu, ui.message_handle()))
}

#[inline(always)]
unsafe fn build_menu_item<S: Clone+Into<String>, ID: Clone+Hash>(ui: &Ui<ID>, t: &MenuItemT<S, ID>) -> Result<(HMENU, UINT), Error> {
    use user32::{AppendMenuW, CreateMenu, GetMenu, SetMenu, DrawMenuBar};
    use winapi::{MF_STRING, UINT_PTR};
    use low::other_helper::to_utf16;
    use low::menu_helper::enable_menuitem;
    
    let ph_result = ui.handle_of(&t.parent);
    if ph_result.is_err() { return Err(ph_result.err().unwrap()); }

    match ph_result.unwrap() {
        AnyHandle::HWND(parent_h) => {
            let mut menubar = GetMenu(parent_h);
            if menubar.is_null() {
                // If the window do not have a menu bar, create one
                menubar = CreateMenu();
                SetMenu(parent_h, menubar);
            }

            MENU_ITEMS_ID += 1;
            let text = to_utf16(t.text.clone().into().as_ref());
            let ensure_id_stays_the_same = MENU_ITEMS_ID;

            AppendMenuW(menubar, MF_STRING, ensure_id_stays_the_same as UINT_PTR, text.as_ptr());
            enable_menuitem(menubar, None, None, !t.disabled);

            DrawMenuBar(parent_h); // Draw the menu bar to make sure the changes are visible

            // WATCH OUT HERE!!! Calling `DrawMenuBar` (or maybe AppendMenuW) corrupted the MENU_ITEMS_ID value (which in turn f* the whole menuitem system)
            // Saving the id in its own little variable saved the day

            Ok( (menubar, ensure_id_stays_the_same) )
        },
        AnyHandle::HMENU(parent_h) => {
            let text = to_utf16(t.text.clone().into().as_ref());
            MENU_ITEMS_ID += 1;
            AppendMenuW(parent_h, MF_STRING, MENU_ITEMS_ID as UINT_PTR, text.as_ptr());
            enable_menuitem(parent_h, None, None, !t.disabled);
            Ok( (parent_h, MENU_ITEMS_ID) )
        },
        h => Err( Error::BadParent(format!("A menu item parent must be a Menu or a Window. Got {:?}", h)) )
    }
}

#[inline(always)]
unsafe fn build_separator<ID: Clone+Hash>(ui: &Ui<ID>, t: &SeparatorT<ID>) -> Result<(HMENU, UINT), Error> {
    use user32::AppendMenuW;
    use winapi::MENUITEMINFOW;
    use low::defs::{MF_SEPARATOR, SetMenuItemInfoW, GetMenuItemCount, MIIM_ID};
    use std::ptr;
    
    let ph_result = ui.handle_of(&t.parent);
    if ph_result.is_err() { return Err(ph_result.err().unwrap()); }

    match ph_result.unwrap() {
        AnyHandle::HMENU(parent_h) => {
            MENU_ITEMS_ID += 1;
            let ensure_id_stays_the_same = MENU_ITEMS_ID;

            // MF_SEPARATOR ignore the lpNewItem and uIDNewItem parameters, so they must be setted using SetMenuItemInfo
            AppendMenuW(parent_h, MF_SEPARATOR, 0, ptr::null());

            // Set the unique id of the separator
            let pos = GetMenuItemCount(parent_h) - 1;
            let mut info = MENUITEMINFOW{ 
                cbSize: mem::size_of::<MENUITEMINFOW>() as UINT,
                fMask: MIIM_ID, fType: 0, fState: 0,
                wID: ensure_id_stays_the_same as UINT,
                hSubMenu: ptr::null_mut(), hbmpChecked: ptr::null_mut(),
                hbmpUnchecked: ptr::null_mut(), dwItemData: 0, dwTypeData: ptr::null_mut(),
                cch: 0, hbmpItem: ptr::null_mut()
            };

            SetMenuItemInfoW(parent_h, pos as UINT, true as BOOL, &mut info);

            Ok( (parent_h, ensure_id_stays_the_same) )
        },
        h => Err( Error::BadParent(format!("A separator parent must be a Menu. Got {:?}", h)) )
    }
}