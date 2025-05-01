// Import required libraries
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, window};
use std::cell::RefCell;
use std::rc::Rc;

// Import local modules
use crate::rendering::Renderer;
use crate::entity::{Player, Enemy, Bullet, Entity};
use crate::input::InputHandler;

/// Game state representation
#[derive(PartialEq, Eq, Clone)]
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
        
        // Create a clone for the game instance to use outside the closure
        let game_instance = game.clone();
        
        // Define the game loop closure that will be called on each animation frame
        let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let g = f.clone();
        
        // This is the actual game loop function that will run on each animation frame
        *g.borrow_mut() = Some(Closure::wrap(Box::new({
            // Clone game for the closure
            let game_loop = game.clone();
            move || {
                let mut game = game_loop.borrow_mut();
                
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
            }
        }) as Box<dyn FnMut()>));
        
        // Start the game loop by requesting the first animation frame
        let window = window().expect("no global window exists");
        let animation_id = window
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("failed to request animation frame");
        
        // Store game in a global to prevent it from being dropped
        // This needs to be done because we're using the 'self' value
        let closure = Closure::wrap(Box::new(move || {
            let _ = game_instance; // Keep the reference alive
        }) as Box<dyn FnMut()>);
        closure.forget(); // Leak the closure to keep game alive
        
        // We can't update the animation_id in self because we've already moved it
        // Instead, we update it in the Rc<RefCell<Game>> we created
        game_instance.borrow_mut().animation_id = Some(animation_id);
        
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
        
        // Get player position for enemy aiming
        let (player_x, player_y) = self.player.position();
        
        // Update enemies and check if they should shoot
        for enemy in &mut self.enemies {
            enemy.update();
            
            // Enemies have a chance to shoot if they're in view and ready
            if enemy.can_shoot() && 
               enemy.position().1 > 0.0 && // Only shoot if on screen
               js_sys::Math::random() < 0.01 { // 1% chance per frame
                
                let (enemy_x, enemy_y) = enemy.position();
                
                // Create a new bullet based on enemy type
                let bullet = match enemy.enemy_type() {
                    crate::entity::EnemyType::Basic => {
                        // Basic enemies shoot straight down
                        crate::entity::Bullet::new_enemy_bullet(enemy_x, enemy_y + 20.0)
                    },
                    crate::entity::EnemyType::Zigzag => {
                        // Zigzag enemies shoot in random directions
                        let angle = js_sys::Math::random() * std::f64::consts::PI * 2.0;
                        let dx = js_sys::Math::cos(angle);
                        let dy = js_sys::Math::sin(angle);
                        crate::entity::Bullet::new_aimed_enemy_bullet(enemy_x, enemy_y + 10.0, dx, dy)
                    },
                    crate::entity::EnemyType::Follower => {
                        // Follower enemies aim at the player
                        let (dx, dy) = enemy.aim_at_player(player_x, player_y);
                        crate::entity::Bullet::new_aimed_enemy_bullet(enemy_x, enemy_y + 10.0, dx, dy)
                    }
                };
                
                // Add the bullet to the game
                self.bullets.push(bullet);
            }
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
        // Get current input state
        let input_state = self.input_handler.get_state();
        
        // Handle game state changes first
        if input_state.space_pressed {
            match self.state {
                GameState::MainMenu => {
                    // Start the game when space is pressed at main menu
                    self.state = GameState::Playing;
                    self.needs_redraw = true;
                },
                GameState::GameOver => {
                    // Reset and restart the game
                    self.reset_game();
                    self.state = GameState::Playing;
                    self.needs_redraw = true;
                },
                _ => {} // Do nothing for other states
            }
        }
        
        if input_state.escape_pressed && self.state == GameState::Playing {
            // Pause the game
            self.state = GameState::MainMenu;
            self.needs_redraw = true;
            return;
        }
        
        // Only process movement and shooting when the game is active
        if self.state == GameState::Playing {
            // Process player movement
            self.player.handle_input(&input_state);
            
            // Handle shooting - check if player can shoot and space is pressed
            if input_state.space_pressed && self.player.can_shoot() {
                // Get player position
                let (player_x, player_y) = self.player.position();
                
                // Create a new bullet just above the player
                let bullet = crate::entity::Bullet::new_player_bullet(player_x, player_y - 20.0);
                self.bullets.push(bullet);
                
                // Maybe add sound effect here in the future?
            }
        }
    }
    
    /// Resets the game to initial state
    fn reset_game(&mut self) {
        // Clear enemies and bullets
        self.enemies.clear();
        self.bullets.clear();
        
        // Reset score
        self.score = 0;
        
        // Reset player position
        let player_x = self.canvas.width() as f64 / 2.0;
        let player_y = self.canvas.height() as f64 * 0.9;
        self.player = crate::entity::Player::new(player_x, player_y);
        
        // Mark for redraw
        self.needs_redraw = true;
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
        // Only spawn enemies during gameplay
        if self.state != GameState::Playing {
            return;
        }
        
        // Use a simple counter to spawn enemies at intervals
        // This would usually use a timer, but for simplicity we'll use a random chance
        if js_sys::Math::random() < 0.02 { // 2% chance per frame to spawn an enemy
            // Get canvas width for spawning enemies across the screen
            let canvas_width = self.canvas.width() as f64;
            
            // Random x position for the enemy
            let x = js_sys::Math::random() * (canvas_width - 40.0) + 20.0;
            
            // Create enemy at the top of the screen
            let enemy_type = match (js_sys::Math::random() * 3.0) as u32 {
                0 => crate::entity::EnemyType::Basic,
                1 => crate::entity::EnemyType::Zigzag,
                _ => crate::entity::EnemyType::Follower,
            };
            
            // Create and add the enemy
            let enemy = crate::entity::Enemy::new(x, -30.0, enemy_type);
            self.enemies.push(enemy);
        }
        
        // Occasionally spawn a formation of enemies
        if self.enemies.len() < 5 && js_sys::Math::random() < 0.005 { // 0.5% chance for a formation
            self.spawn_enemy_formation();
        }
    }
    
    /// Spawns a formation of enemies
    fn spawn_enemy_formation(&mut self) {
        let canvas_width = self.canvas.width() as f64;
        let formation_type = (js_sys::Math::random() * 3.0) as u32;
        
        match formation_type {
            // V formation
            0 => {
                let center_x = canvas_width / 2.0;
                for i in -2..=2i32 {
                    let x = center_x + (i as f64 * 40.0);
                    let y = -30.0 - (i.abs() as f64 * 20.0);
                    let enemy = crate::entity::Enemy::new(
                        x, 
                        y, 
                        crate::entity::EnemyType::Basic
                    );
                    self.enemies.push(enemy);
                }
            },
            
            // Line formation
            1 => {
                let spacing = canvas_width / 6.0;
                for i in 1..=5 {
                    let x = i as f64 * spacing;
                    let enemy = crate::entity::Enemy::new(
                        x, 
                        -30.0, 
                        crate::entity::EnemyType::Zigzag
                    );
                    self.enemies.push(enemy);
                }
            },
            
            // Diamond formation
            _ => {
                let center_x = canvas_width / 2.0;
                let positions = [
                    (center_x, -70.0),           // Top
                    (center_x - 40.0, -30.0),    // Left
                    (center_x + 40.0, -30.0),    // Right
                    (center_x, 10.0),            // Bottom
                ];
                
                for (x, y) in positions.iter() {
                    let enemy = crate::entity::Enemy::new(
                        *x, 
                        *y, 
                        crate::entity::EnemyType::Follower
                    );
                    self.enemies.push(enemy);
                }
            }
        }
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
        
        // Draw UI - clone the state to avoid ownership problems
        self.renderer.draw_ui(self.score, self.state.clone())?;
        
        Ok(())
    }
    
    /// Handles game over state
    fn game_over(&mut self) {
        self.state = GameState::GameOver;
        self.needs_redraw = true;
        
        // TODO: Implement game over screen and logic
    }
} 