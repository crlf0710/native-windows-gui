/*!
    A simple text box control

    This is basically a multiline text input but I want to differenciate the two controls
    because they have a few difference and I want to keep each control in its own file with
    no superclass.
*/

use std::hash::Hash;
use std::any::TypeId;

use winapi::{HWND, HFONT, WPARAM};
use user32::SendMessageW;

use ui::Ui;
use controls::{Control, ControlT, ControlType, AnyHandle};
use error::Error;

/**
    A template that creates a multi line textinput control

    Control specific events:  
    `textbox::ValueChanged, textbox::Focus` 

    Members:  
    • `text`: The text of the textbox  
    • `position`: The start position of the textbox  
    • `size`: The start size of the textbox  
    • `visible`: If the textbox should be visible to the user   
    • `disabled`: If the user can or can't click on the textbox  
    • `readonly`: If the user can copty the text but can't edit the textbox content  
    • `limit`: The maximum number of characters that the control can hold  
    • `scrollbars`: A tuple to defined whether to show scrollbars or not (show horizontal, show vertical)
    • `parent`: The textbox parent  
    • `font`: The textbox font. If None, use the system default  
*/
#[derive(Clone)]
pub struct TextBoxT<S1: Clone+Into<String>, ID: Hash+Clone> {
    pub text: S1,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub limit: u32,
    pub scrollbars: (bool, bool),
    pub parent: ID,
    pub font: Option<ID>,
}

impl<S1: Clone+Into<String>, ID: Hash+Clone> ControlT<ID> for TextBoxT<S1, ID> {
    fn type_id(&self) -> TypeId { TypeId::of::<TextBox>() }

    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        use low::window_helper::{WindowParams, build_window, set_window_font_raw, handle_of_window, handle_of_font};
        use low::defs::{ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_READONLY, EM_LIMITTEXT, ES_MULTILINE};
        use winapi::{DWORD, WS_VISIBLE, WS_DISABLED, WS_CHILD, WS_BORDER, WS_HSCROLL, WS_VSCROLL};

        let flags: DWORD = WS_CHILD | WS_BORDER | ES_AUTOHSCROLL | ES_MULTILINE | ES_AUTOVSCROLL |
        if self.readonly { ES_READONLY } else { 0 } |
        if self.visible  { WS_VISIBLE }  else { 0 } |
        if self.scrollbars.0 { WS_HSCROLL } else { 0 } |
        if self.scrollbars.1 { WS_VSCROLL } else { 0 } |
        if self.disabled { WS_DISABLED } else { 0 };

        // Get the parent handle
        let parent = match handle_of_window(ui, &self.parent, "The parent of a textinput must be a window-like control.") {
            Ok(h) => h,
            Err(e) => { return Err(e); }
        };

        // Get the font handle (if any)
        let font_handle: Option<HFONT> = match self.font.as_ref() {
            Some(font_id) => 
                match handle_of_font(ui, &font_id, "The font of a button must be a font resource.") {
                    Ok(h) => Some(h),
                    Err(e) => { return Err(e); }
                },
            None => None
        };

        let params = WindowParams {
            title: self.text.clone().into(),
            class_name: "EDIT",
            position: self.position.clone(),
            size: self.size.clone(),
            flags: flags,
            ex_flags: Some(0),
            parent: parent
        };

        match unsafe{ build_window(params) } {
            Ok(h) => {
                unsafe{ 
                    set_window_font_raw(h, font_handle, true); 
                    SendMessageW(h, EM_LIMITTEXT, self.limit as WPARAM, 0);
                };

                Ok( Box::new(TextBox{handle: h}) )
            },
            Err(e) => Err(Error::System(e))
        }
    }
}

/**
    A multi line textinput control
*/
pub struct TextBox {
    handle: HWND
}

impl TextBox {

    /// Set or unset the readonly status on the control
    pub fn set_readonly(&self, readonly: bool) {
        use low::window_helper::{set_window_long, get_window_long};
        use low::defs::ES_READONLY;
        use winapi::GWL_STYLE;

        let old_style = get_window_long(self.handle, GWL_STYLE) as usize;
        if readonly {
            set_window_long(self.handle, GWL_STYLE, old_style|(ES_READONLY as usize));
        } else {
            set_window_long(self.handle, GWL_STYLE, old_style&(!ES_READONLY as usize) );
        }
    }
    
    /// Return `true` if the user cannot edit the content of the control or `false` if the user can
    pub fn get_readonly(&self) -> bool {
        use low::window_helper::get_window_long;
        use low::defs::ES_READONLY;
        use winapi::GWL_STYLE;

        let style = get_window_long(self.handle, GWL_STYLE) as u32;

        (style & ES_READONLY) == ES_READONLY
    }

    /// Set the maximum number of characters that the control can hold
    pub fn set_limit(&self, limit: u32) {
        use low::defs::EM_LIMITTEXT;
        unsafe{ SendMessageW(self.handle, EM_LIMITTEXT, limit as WPARAM, 0); }
    }

    /// Return the maximum number of characters that the control can hold
    pub fn get_limit(&self) -> u32 {
        use low::defs::EM_GETLIMITTEXT;
        unsafe{ SendMessageW(self.handle, EM_GETLIMITTEXT, 0, 0) as u32 }
    }

    pub fn get_text(&self) -> String { unsafe{ ::low::window_helper::get_window_text(self.handle) } }
    pub fn set_text<'a>(&self, text: &'a str) { unsafe{ ::low::window_helper::set_window_text(self.handle, text); } }
    pub fn get_visibility(&self) -> bool { unsafe{ ::low::window_helper::get_window_visibility(self.handle) } }
    pub fn set_visibility(&self, visible: bool) { unsafe{ ::low::window_helper::set_window_visibility(self.handle, visible); }}
    pub fn get_position(&self) -> (i32, i32) { unsafe{ ::low::window_helper::get_window_position(self.handle) } }
    pub fn set_position(&self, x: i32, y: i32) { unsafe{ ::low::window_helper::set_window_position(self.handle, x, y); }}
    pub fn get_size(&self) -> (u32, u32) { unsafe{ ::low::window_helper::get_window_size(self.handle) } }
    pub fn set_size(&self, w: u32, h: u32) { unsafe{ ::low::window_helper::set_window_size(self.handle, w, h, true); } }
    pub fn get_enabled(&self) -> bool { unsafe{ ::low::window_helper::get_window_enabled(self.handle) } }
    pub fn set_enabled(&self, e:bool) { unsafe{ ::low::window_helper::set_window_enabled(self.handle, e); } }
    pub fn get_font<ID: Hash+Clone>(&self, ui: &Ui<ID>) -> Option<ID> { unsafe{ ::low::window_helper::get_window_font(self.handle, ui) } }
    pub fn set_font<ID: Hash+Clone>(&self, ui: &Ui<ID>, f: Option<&ID>) -> Result<(), Error> { unsafe{ ::low::window_helper::set_window_font(self.handle, ui, f) } }
    pub fn update(&self) { unsafe{ ::low::window_helper::update(self.handle); } }
    pub fn focus(&self) { unsafe{ ::user32::SetFocus(self.handle); } }
}

impl Control for TextBox {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HWND(self.handle)
    }

    fn control_type(&self) -> ControlType { 
        ControlType::TextBox 
    }

    fn free(&mut self) {
        use user32::DestroyWindow;
        unsafe{ DestroyWindow(self.handle) };
    }

}