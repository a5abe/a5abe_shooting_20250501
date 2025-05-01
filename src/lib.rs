// Import necessary libraries for WebAssembly and browser interaction
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

// Import modules (we will create these next)
mod game;
mod rendering;
mod entity;
mod input;

// This function is called when the WASM module is loaded
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Set up panic hook for better error messages during development
    console_error_panic_hook::set_once();
    
    // Initialize the game
    let window = web_sys::window().expect("No global window exists");
    let document = window.document().expect("No document exists on window");
    
    // Get the canvas element from HTML
    let canvas = document
        .get_element_by_id("game-canvas")
        .ok_or_else(|| JsValue::from_str("Cannot find canvas element with id 'game-canvas'"))?
        .dyn_into::<HtmlCanvasElement>()?;
    
    // Initialize the game with the canvas
    let game_instance = game::Game::new(canvas)?;
    
    // Start the game loop
    // Note we're consuming game_instance here because the start method now takes ownership
    game_instance.start()?;
    
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
