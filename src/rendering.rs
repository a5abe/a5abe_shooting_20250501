// Import required libraries
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// Import game modules
use crate::entity::{Player, Enemy, Bullet};
use crate::game::GameState;

/// Colors used in the game
struct Colors {
    background: &'static str,
    player: &'static str,
    enemy: &'static str,
    player_bullet: &'static str,
    enemy_bullet: &'static str,
    ui_text: &'static str,
}

impl Colors {
    /// Creates a new color palette
    fn new() -> Self {
        Self {
            background: "#000022",
            player: "#00FFFF",
            enemy: "#FF3366",
            player_bullet: "#88FFFF",
            enemy_bullet: "#FF8888",
            ui_text: "#FFFFFF",
        }
    }
}

/// Renderer handles all drawing operations in the game
pub struct Renderer {
    /// Canvas 2D rendering context
    context: CanvasRenderingContext2d,
    /// Color palette
    colors: Colors,
}

impl Renderer {
    /// Creates a new renderer
    ///
    /// # Arguments
    ///
    /// * `context` - The 2D canvas rendering context
    ///
    /// # Returns
    ///
    /// A Result containing the new Renderer or an error
    pub fn new(context: CanvasRenderingContext2d) -> Result<Self, JsValue> {
        // Set up initial canvas properties
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.set_font("16px Arial");
        
        Ok(Self {
            context,
            colors: Colors::new(),
        })
    }
    
    /// Clears the canvas for a new frame
    pub fn clear(&self) -> Result<(), JsValue> {
        // Get canvas dimensions
        let width = self.context.canvas().unwrap().width() as f64;
        let height = self.context.canvas().unwrap().height() as f64;
        
        // Clear the entire canvas
        self.context.save();
        self.context.set_fill_style(&JsValue::from_str(self.colors.background));
        self.context.fill_rect(0.0, 0.0, width, height);
        self.context.restore();
        
        Ok(())
    }
    
    /// Draws the game background
    pub fn draw_background(&self) -> Result<(), JsValue> {
        // Currently just a basic background
        // Could be expanded to have stars or other decorative elements
        Ok(())
    }
    
    /// Draws the player ship
    ///
    /// # Arguments
    ///
    /// * `player` - Reference to the player object
    pub fn draw_player(&self, player: &Player) -> Result<(), JsValue> {
        self.context.save();
        
        // Set player color
        self.context.set_fill_style(&JsValue::from_str(self.colors.player));
        
        // Draw player as a triangle
        self.context.begin_path();
        
        // Get player position and size
        let (x, y) = player.position();
        let size = player.size();
        
        // Draw player triangle (pointing upwards)
        self.context.move_to(x, y - size / 2.0); // Top point
        self.context.line_to(x - size / 2.0, y + size / 2.0); // Bottom left
        self.context.line_to(x + size / 2.0, y + size / 2.0); // Bottom right
        self.context.close_path();
        
        self.context.fill();
        self.context.restore();
        
        Ok(())
    }
    
    /// Draws an enemy ship
    ///
    /// # Arguments
    ///
    /// * `enemy` - Reference to the enemy object
    pub fn draw_enemy(&self, enemy: &Enemy) -> Result<(), JsValue> {
        self.context.save();
        
        // Set enemy color
        self.context.set_fill_style(&JsValue::from_str(self.colors.enemy));
        
        // Get enemy position and size
        let (x, y) = enemy.position();
        let size = enemy.size();
        
        // Draw enemy as an inverted triangle (pointing downwards)
        self.context.begin_path();
        self.context.move_to(x, y + size / 2.0); // Bottom point
        self.context.line_to(x - size / 2.0, y - size / 2.0); // Top left
        self.context.line_to(x + size / 2.0, y - size / 2.0); // Top right
        self.context.close_path();
        
        self.context.fill();
        self.context.restore();
        
        Ok(())
    }
    
    /// Draws a bullet
    ///
    /// # Arguments
    ///
    /// * `bullet` - Reference to the bullet object
    pub fn draw_bullet(&self, bullet: &Bullet) -> Result<(), JsValue> {
        self.context.save();
        
        // Set bullet color based on source (player or enemy)
        let color = if bullet.is_from_player() {
            self.colors.player_bullet
        } else {
            self.colors.enemy_bullet
        };
        
        self.context.set_fill_style(&JsValue::from_str(color));
        
        // Get bullet position and size
        let (x, y) = bullet.position();
        let size = bullet.size();
        
        // Draw bullet as a small circle
        self.context.begin_path();
        let _ = self.context.arc(x, y, size / 2.0, 0.0, std::f64::consts::PI * 2.0);
        self.context.fill();
        
        self.context.restore();
        
        Ok(())
    }
    
    /// Draws the UI elements (score, game state)
    ///
    /// # Arguments
    ///
    /// * `score` - Current game score
    /// * `state` - Current game state
    pub fn draw_ui(&self, score: u32, state: GameState) -> Result<(), JsValue> {
        self.context.save();
        
        // Set text properties
        self.context.set_fill_style(&JsValue::from_str(self.colors.ui_text));
        self.context.set_font("20px Arial");
        
        // Draw score in the top-left corner
        let score_text = format!("Score: {}", score);
        let _ = self.context.fill_text(&score_text, 70.0, 30.0);
        
        // Draw game state information if not in Playing state
        match state {
            GameState::MainMenu => {
                self.context.set_font("36px Arial");
                let _ = self.context.fill_text("SPACE SHOOTER", self.canvas_width() / 2.0, self.canvas_height() / 3.0);
                
                self.context.set_font("24px Arial");
                let _ = self.context.fill_text("Press SPACE to start", self.canvas_width() / 2.0, self.canvas_height() / 2.0);
            },
            GameState::Paused => {
                self.context.set_font("36px Arial");
                let _ = self.context.fill_text("PAUSED", self.canvas_width() / 2.0, self.canvas_height() / 2.0);
                
                self.context.set_font("24px Arial");
                let _ = self.context.fill_text("Press SPACE to continue", self.canvas_width() / 2.0, self.canvas_height() / 2.0 + 40.0);
            },
            GameState::GameOver => {
                self.context.set_font("36px Arial");
                let _ = self.context.fill_text("GAME OVER", self.canvas_width() / 2.0, self.canvas_height() / 3.0);
                
                let score_text = format!("Final Score: {}", score);
                let _ = self.context.fill_text(&score_text, self.canvas_width() / 2.0, self.canvas_height() / 2.0);
                
                self.context.set_font("24px Arial");
                let _ = self.context.fill_text("Press SPACE to restart", self.canvas_width() / 2.0, self.canvas_height() / 2.0 + 60.0);
            },
            GameState::Playing => {
                // Nothing extra to draw during gameplay
            }
        }
        
        self.context.restore();
        
        Ok(())
    }
    
    /// Helper function to get canvas width
    fn canvas_width(&self) -> f64 {
        self.context.canvas().unwrap().width() as f64
    }
    
    /// Helper function to get canvas height
    fn canvas_height(&self) -> f64 {
        self.context.canvas().unwrap().height() as f64
    }
} 