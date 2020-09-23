use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
    rc::Rc,
    cell::{Cell, RefCell},
};
use twox_hash::XxHash32;
use wasm_bindgen::JsValue;
use crate::input::listener::EventListener;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum InputState
{
    /// Key/Button is not pressed
    Up,
    /// Key/Button is pressed
    Down,
    /// Key/Button is being held down
    Repeating
}

/// Stores current states from user key and mouse input
/// This should NOT be used to replace an on<*> event listener,
/// this is just for being able to query the current input state
/// of a keyboard/mouse
pub struct InputStateListener
{
    target: web_sys::EventTarget,
    listeners: Vec<EventListener>,

    keys: Rc<RefCell<HashMap<String, InputState, BuildHasherDefault<XxHash32>>>>,

    mouse_buttons: Rc<RefCell<[InputState; 5]>>,
    curr_mouse_pos: Rc<Cell<(i32, i32)>>,
    last_mouse_pos: Rc<Cell<(i32, i32)>>
}

impl InputStateListener
{
    pub fn new(target: &web_sys::EventTarget) -> Result<InputStateListener, JsValue>
    {
        let mut manager = InputStateListener
        {
            target: target.clone(),
            listeners: Vec::with_capacity(5),

            keys: Default::default(),

            mouse_buttons: Rc::new(RefCell::new([InputState::Up; 5])),
            curr_mouse_pos: Rc::new(Cell::new((0, 0))),
            last_mouse_pos: Rc::new(Cell::new((0, 0))),
        };

        // keydown listener
        let ev = EventListener::new(&manager.target, "keydown",
                                    {
                                        clone!(manager.keys);
                                        move |event: web_sys::KeyboardEvent|
                                            {
                                                if event.repeat()
                                                {
                                                    keys.borrow_mut().insert(event.key(), InputState::Repeating);
                                                }
                                                else
                                                {
                                                    keys.borrow_mut().insert(event.key(), InputState::Down);
                                                }
                                            }
                                    }).expect("keydown event listener");
        manager.listeners.push(ev);

        // keyup listener
        let ev = EventListener::new(&manager.target, "keyup",
                                    {
                                        clone!(manager.keys);
                                        move |event: web_sys::KeyboardEvent| { keys.borrow_mut().insert(event.key(), InputState::Up); }
                                    }).expect("keyup event listener");
        manager.listeners.push(ev);

        // mouseup listener
        let ev = EventListener::new(&manager.target, "mouseup",
                                    {
                                        clone!(manager.mouse_buttons);
                                        move |event: web_sys::MouseEvent|
                                            {
                                                let button = event.button();
                                                if button <= 4
                                                {
                                                    mouse_buttons.borrow_mut()[button as usize] = InputState::Up;
                                                }
                                            }
                                    }).expect("mouseup event listener");
        manager.listeners.push(ev);

        // mousedown listener
        let ev = EventListener::new(&manager.target, "mousedown",
                                    {
                                        clone!(manager.mouse_buttons);
                                        move |event: web_sys::MouseEvent|
                                            {
                                                let button = event.button();
                                                if button <= 4
                                                {
                                                    mouse_buttons.borrow_mut()[button as usize] = InputState::Down;
                                                }
                                            }
                                    }).expect("mousedown event listener");
        manager.listeners.push(ev);

        // mousemove listener
        let ev = EventListener::new(&manager.target, "mousemove",
                                    {
                                        clone!(manager.curr_mouse_pos, manager.last_mouse_pos);
                                        move |event: web_sys::MouseEvent|
                                            {
                                                let last = curr_mouse_pos.replace((event.offset_x(), event.offset_y()));
                                                last_mouse_pos.set(last);
                                            }
                                    }).expect("mousemove event listener");
        manager.listeners.push(ev);

        Ok(manager)
    }

    /// Get the current state of `key`
    #[allow(dead_code)]
    pub fn key_state(&self, key: &str) -> InputState
    {
        // If `key` is not in the internal hashmap, it has not
        // been pressed yet since it would otherwise have been entered
        // into the internal hashmap via the "keydown" event listener,
        // so it either doesn't exist or is "Up"
        *self.keys.borrow().get(key).unwrap_or(&InputState::Up)
    }

    /// Get the current state of `button`
    /// Will panic if `button` is not a valid mouse button
    #[allow(dead_code)]
    pub fn mouse_btn_state(&self, button: usize) -> InputState
    {
        self.mouse_buttons.borrow()[button]
    }

    /// Current mouse position
    /// 0,0 is top left of `target` element
    #[allow(dead_code)]
    pub fn curr_mouse_pos(&self) -> (i32, i32)
    {
        self.curr_mouse_pos.get()
    }

    /// Last mouse position
    /// 0,0 is top left of `target` element
    #[allow(dead_code)]
    pub fn last_mouse_pos(&self) -> (i32, i32)
    {
        self.last_mouse_pos.get()
    }
}