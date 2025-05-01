// Input module for handling keyboard input

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, EventTarget, window};
use std::cell::RefCell;
use std::rc::Rc;

/// Represents the state of player input
#[derive(Default, Clone)]
pub struct InputState {
    /// Whether the left arrow or A key is pressed
    pub left_pressed: bool,
    /// Whether the right arrow or D key is pressed
    pub right_pressed: bool,
    /// Whether the up arrow or W key is pressed
    pub up_pressed: bool,
    /// Whether the down arrow or S key is pressed
    pub down_pressed: bool,
    /// Whether the space key is pressed
    pub space_pressed: bool,
    /// Whether the escape key is pressed
    pub escape_pressed: bool,
}

/// Handles keyboard input for the game
pub struct InputHandler {
    /// The current input state
    input_state: Rc<RefCell<InputState>>,
    /// Keyboard event handlers
    _event_handlers: Vec<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl InputHandler {
    /// Creates a new input handler and registers event listeners
    pub fn new() -> Result<Self, JsValue> {
        // Get the window
        let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;
        
        // Create a shared input state
        let input_state = Rc::new(RefCell::new(InputState::default()));
        
        // Create closures for keydown and keyup events
        let keydown_state = input_state.clone();
        let keyup_state = input_state.clone();
        
        // Create keydown handler
        let keydown_handler = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let mut state = keydown_state.borrow_mut();
            match event.key().as_str() {
                "ArrowLeft" | "a" | "A" => state.left_pressed = true,
                "ArrowRight" | "d" | "D" => state.right_pressed = true,
                "ArrowUp" | "w" | "W" => state.up_pressed = true,
                "ArrowDown" | "s" | "S" => state.down_pressed = true,
                " " => state.space_pressed = true,
                "Escape" => state.escape_pressed = true,
                _ => {}
            }
            // Prevent default actions for arrow keys (e.g., scrolling)
            if event.key().starts_with("Arrow") || event.key() == " " {
                event.prevent_default();
            }
        }) as Box<dyn FnMut(_)>);
        
        // Create keyup handler
        let keyup_handler = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let mut state = keyup_state.borrow_mut();
            match event.key().as_str() {
                "ArrowLeft" | "a" | "A" => state.left_pressed = false,
                "ArrowRight" | "d" | "D" => state.right_pressed = false,
                "ArrowUp" | "w" | "W" => state.up_pressed = false,
                "ArrowDown" | "s" | "S" => state.down_pressed = false,
                " " => state.space_pressed = false,
                "Escape" => state.escape_pressed = false,
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);
        
        // Add event listeners
        let event_target: &EventTarget = document.as_ref();
        event_target.add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())?;
        event_target.add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref())?;
        
        // Store handlers so they aren't dropped (which would remove the listeners)
        let event_handlers = vec![keydown_handler, keyup_handler];
        
        Ok(Self {
            input_state,
            _event_handlers: event_handlers,
        })
    }
    
    /// Gets the current input state
    pub fn get_state(&self) -> InputState {
        self.input_state.borrow().clone()
    }
} 