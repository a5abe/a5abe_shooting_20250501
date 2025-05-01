// Import required libraries
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, window};
use std::cell::RefCell;
use std::rc::Rc;

// Import local modules
use crate::rendering::Renderer;
use crate::entity::{Player, Enemy, Bullet};
use crate::input::InputHandler;

/// Game state representation
#[derive(PartialEq, Eq)]
pub enum GameState {
    /// Game is in main menu
    MainMenu,
    /// Game is actively running
    Playing,
    /// Game is paused
    Paused,
    /// Game is over (player died)
    GameOver,
}

/// Main game class that manages the game loop and state
pub struct Game {
    /// The HTML canvas element 
    canvas: HtmlCanvasElement,
    /// The rendering context
    context: CanvasRenderingContext2d,
    /// The game renderer
    renderer: Renderer,
    /// Current game state
    state: GameState,
    /// Player entity
    player: Player,
    /// List of active enemies
    enemies: Vec<Enemy>,
    /// List of active bullets
    bullets: Vec<Bullet>,
    /// Current game score
    score: u32,
    /// Animation frame ID for the game loop
    animation_id: Option<i32>,
    /// Input handler
    input_handler: InputHandler,
    /// Flag indicating if the game state is dirty and needs redrawing
    needs_redraw: bool,
}

impl Game {
    /// Creates a new Game instance
    ///
    /// # Arguments
    ///
    /// * `canvas` - The HTML canvas element to render the game on
    ///
    /// # Returns
    ///
    /// A Result containing the new Game instance or an error
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // Set up canvas context
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        // Create input handler
        let input_handler = InputHandler::new()?;
        
        // Create renderer
        let renderer = Renderer::new(context.clone())?;
        
        // Create player at the bottom center of the screen
        let player_x = canvas.width() as f64 / 2.0;
        let player_y = canvas.height() as f64 * 0.9;
        let player = Player::new(player_x, player_y);
        
        Ok(Self {
            canvas,
            context,
            renderer,
            state: GameState::MainMenu,
            player,
            enemies: Vec::new(),
            bullets: Vec::new(),
            score: 0,
            animation_id: None,
            input_handler,
            needs_redraw: true,
        })
    }
    
    /// Starts the game loop
    pub fn start(self) -> Result<(), JsValue> {
        // Create a shared reference to the game that can be modified in the animation loop
        let game = Rc::new(RefCell::new(self));
        
        // Define the game loop closure that will be called on each animation frame
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        
        // This is the actual game loop function that will run on each animation frame
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let mut game = game.borrow_mut();
            
            // If game is running, update the game state
            if game.state == GameState::Playing {
                // Update game logic
                game.update();
                
                // Mark for redraw
                game.needs_redraw = true;
            }
            
            // Render the game if needed
            if game.needs_redraw {
                let _ = game.render(); // We use let _ to explicitly ignore the result
                game.needs_redraw = false;
            }
            
            // Request the next animation frame
            let window = window().expect("no global window exists");
            game.animation_id = Some(window
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("failed to request animation frame"));
        }) as Box<dyn FnMut()>));
        
        // Start the game loop by requesting the first animation frame
        let window = window().expect("no global window exists");
        let animation_id = window
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("failed to request animation frame");
        
        // Store game in a global to prevent it from being dropped
        // This needs to be done because we're using the 'self' value
        let game_instance = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            let _ = game_instance; // Keep the reference alive
        }) as Box<dyn FnMut()>);
        closure.forget(); // Leak the closure to keep game alive
        
        // We can't update the animation_id in self because we've already moved it
        // Instead, we update it in the Rc<RefCell<Game>> we created
        game.borrow_mut().animation_id = Some(animation_id);
        
        Ok(())
    }
    
    /// Updates the game state for one frame
    fn update(&mut self) {
        // Process inputs
        self.handle_input();
        
        // Update player position
        self.player.update();
        
        // Update bullets
        for bullet in &mut self.bullets {
            bullet.update();
        }
        
        // Update enemies
        for enemy in &mut self.enemies {
            enemy.update();
        }
        
        // Check for collisions
        self.check_collisions();
        
        // Remove bullets that are out of bounds
        self.bullets.retain(|bullet| bullet.is_active());
        
        // Remove enemies that are out of bounds or destroyed
        self.enemies.retain(|enemy| enemy.is_active());
        
        // Spawn new enemies if needed
        self.spawn_enemies();
    }
    
    /// Handles player input
    fn handle_input(&mut self) {
        // This will be implemented in the InputHandler
        let input_state = self.input_handler.get_state();
        self.player.handle_input(&input_state);
    }
    
    /// Checks for collisions between game entities
    fn check_collisions(&mut self) {
        // Check for collisions between bullets and enemies
        // This is a simple implementation and can be optimized
        for bullet in &mut self.bullets {
            if !bullet.is_from_player() {
                continue; // Skip enemy bullets for now
            }
            
            for enemy in &mut self.enemies {
                if bullet.collides_with(enemy) {
                    bullet.deactivate();
                    enemy.take_damage(1);
                    
                    // Add score if enemy is destroyed
                    if !enemy.is_active() {
                        self.score += 100;
                    }
                    
                    break; // Bullet hit something, stop checking
                }
            }
        }
        
        // Check for collisions between player and enemies
        for enemy in &self.enemies {
            if self.player.collides_with(enemy) {
                self.game_over();
                break;
            }
        }
        
        // Check for collisions between player and enemy bullets
        for bullet in &self.bullets {
            if !bullet.is_from_player() && self.player.collides_with(bullet) {
                self.game_over();
                break;
            }
        }
    }
    
    /// Spawns new enemies based on game state
    fn spawn_enemies(&mut self) {
        // TODO: Implement enemy spawning logic
        // This will create enemies at regular intervals based on game difficulty
    }
    
    /// Renders the current game state
    fn render(&self) -> Result<(), JsValue> {
        // Use the renderer to draw the game
        self.renderer.clear()?;
        
        // Draw background
        self.renderer.draw_background()?;
        
        // Draw player
        self.renderer.draw_player(&self.player)?;
        
        // Draw bullets
        for bullet in &self.bullets {
            self.renderer.draw_bullet(bullet)?;
        }
        
        // Draw enemies
        for enemy in &self.enemies {
            self.renderer.draw_enemy(enemy)?;
        }
        
        // Draw UI
        self.renderer.draw_ui(self.score, self.state)?;
        
        Ok(())
    }
    
    /// Handles game over state
    fn game_over(&mut self) {
        self.state = GameState::GameOver;
        self.needs_redraw = true;
        
        // TODO: Implement game over screen and logic
    }
} 