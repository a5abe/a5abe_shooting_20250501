// Entity module contains all game entities like player, enemies, and bullets

// Import required crates
use wasm_bindgen::prelude::*;

// Import our local modules
use crate::input::InputState;

/// A trait for all game entities that can be positioned and sized
pub trait Entity {
    /// Get the position of the entity
    fn position(&self) -> (f64, f64);
    
    /// Get the size of the entity
    fn size(&self) -> f64;
    
    /// Check if the entity is active
    fn is_active(&self) -> bool;
    
    /// Update the entity state for a frame
    fn update(&mut self);
    
    /// Check if this entity collides with another entity
    fn collides_with<T: Entity>(&self, other: &T) -> bool {
        // Don't check collision if either entity is inactive
        if !self.is_active() || !other.is_active() {
            return false;
        }
        
        // Simple circular collision detection
        let (self_x, self_y) = self.position();
        let (other_x, other_y) = other.position();
        
        let self_radius = self.size() / 2.0;
        let other_radius = other.size() / 2.0;
        
        // Calculate distance between centers
        let dx = self_x - other_x;
        let dy = self_y - other_y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Collision occurs if distance is less than sum of radii
        distance < (self_radius + other_radius)
    }
}

/// Represents the player's ship
pub struct Player {
    /// X position
    x: f64,
    /// Y position
    y: f64,
    /// Size of the player ship
    size: f64,
    /// Player movement speed
    speed: f64,
    /// Flag indicating if the player is active
    active: bool,
    /// Player shooting cooldown
    shoot_cooldown: u32,
    /// Current shooting cooldown counter
    current_cooldown: u32,
}

impl Player {
    /// Creates a new player at the specified position
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            size: 30.0, // Player size (ship width/height)
            speed: 5.0, // Player movement speed
            active: true,
            shoot_cooldown: 10, // Wait 10 frames between shots
            current_cooldown: 0,
        }
    }
    
    /// Handle player input
    pub fn handle_input(&mut self, input: &InputState) {
        // Move left/right
        if input.left_pressed {
            self.x -= self.speed;
        }
        if input.right_pressed {
            self.x += self.speed;
        }
        
        // Move up/down
        if input.up_pressed {
            self.y -= self.speed;
        }
        if input.down_pressed {
            self.y += self.speed;
        }
        
        // Check if we can shoot
        if input.space_pressed && self.current_cooldown == 0 {
            // TODO: Create a bullet
            // This will be implemented in the Game struct
            self.current_cooldown = self.shoot_cooldown;
        }
    }
    
    /// Check if player can shoot
    pub fn can_shoot(&self) -> bool {
        self.current_cooldown == 0
    }
}

impl Entity for Player {
    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    
    fn size(&self) -> f64 {
        self.size
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn update(&mut self) {
        // Decrease cooldown if it's active
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
        
        // Additional player update logic could go here
    }
}

/// Represents an enemy ship
pub struct Enemy {
    /// X position
    x: f64,
    /// Y position
    y: f64,
    /// X velocity
    vx: f64,
    /// Y velocity
    vy: f64,
    /// Size of the enemy
    size: f64,
    /// Health points
    hp: u32,
    /// Flag indicating if the enemy is active
    active: bool,
    /// Enemy type (determines behavior)
    enemy_type: EnemyType,
    /// Shoot cooldown
    shoot_cooldown: u32,
    /// Current cooldown counter
    current_cooldown: u32,
}

/// Different enemy types with different behaviors
pub enum EnemyType {
    /// Basic enemy that moves straight down
    Basic,
    /// Enemy that moves in a sine wave pattern
    Zigzag,
    /// Enemy that follows the player
    Follower,
}

impl Enemy {
    /// Creates a new enemy
    pub fn new(x: f64, y: f64, enemy_type: EnemyType) -> Self {
        // Set initial velocities based on enemy type
        let (vx, vy) = match enemy_type {
            EnemyType::Basic => (0.0, 2.0),
            EnemyType::Zigzag => (1.0, 1.5),
            EnemyType::Follower => (0.0, 1.0),
        };
        
        // Set HP based on enemy type
        let hp = match enemy_type {
            EnemyType::Basic => 1,
            EnemyType::Zigzag => 2,
            EnemyType::Follower => 3,
        };
        
        Self {
            x,
            y,
            vx,
            vy,
            size: 25.0,
            hp,
            active: true,
            enemy_type,
            shoot_cooldown: 60, // Shoot every 60 frames (slower than player)
            current_cooldown: (js_sys::Math::random() * 60.0) as u32, // Randomize initial cooldown
        }
    }
    
    /// Take damage and check if destroyed
    pub fn take_damage(&mut self, amount: u32) {
        if self.hp <= amount {
            self.active = false;
        } else {
            self.hp -= amount;
        }
    }
    
    /// Check if enemy can shoot
    pub fn can_shoot(&self) -> bool {
        self.current_cooldown == 0
    }
    
    /// Get enemy position for targeting the player
    pub fn aim_at_player(&self, player_x: f64, player_y: f64) -> (f64, f64) {
        // Calculate direction vector to player
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        
        // Normalize the vector
        let length = (dx * dx + dy * dy).sqrt();
        if length > 0.0 {
            (dx / length, dy / length)
        } else {
            (0.0, 1.0) // Default to shooting downward
        }
    }
}

impl Entity for Enemy {
    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    
    fn size(&self) -> f64 {
        self.size
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn update(&mut self) {
        if !self.active {
            return;
        }
        
        // Update position based on velocity
        match self.enemy_type {
            EnemyType::Basic => {
                // Simply move down
                self.y += self.vy;
            },
            EnemyType::Zigzag => {
                // Move in a sine wave pattern
                self.x += self.vx * (js_sys::Math::sin(self.y * 0.1) * 2.0);
                self.y += self.vy;
            },
            EnemyType::Follower => {
                // Follower logic will be implemented later
                // It needs access to the player position
                self.y += self.vy;
            }
        }
        
        // Update shoot cooldown
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
        
        // Check if enemy is off-screen
        if self.y > 600.0 { // FIXME: Don't hardcode screen height
            self.active = false;
        }
    }
}

/// Represents a bullet fired by player or enemy
pub struct Bullet {
    /// X position
    x: f64,
    /// Y position
    y: f64,
    /// X velocity
    vx: f64,
    /// Y velocity
    vy: f64,
    /// Size of the bullet
    size: f64,
    /// Flag indicating if the bullet is active
    active: bool,
    /// Flag indicating if the bullet was fired by the player
    from_player: bool,
}

impl Bullet {
    /// Creates a new bullet
    pub fn new(x: f64, y: f64, vx: f64, vy: f64, from_player: bool) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            size: if from_player { 8.0 } else { 6.0 }, // Player bullets slightly larger
            active: true,
            from_player,
        }
    }
    
    /// Creates a new player bullet moving upward
    pub fn new_player_bullet(x: f64, y: f64) -> Self {
        Self::new(x, y, 0.0, -8.0, true)
    }
    
    /// Creates a new enemy bullet moving downward
    pub fn new_enemy_bullet(x: f64, y: f64) -> Self {
        Self::new(x, y, 0.0, 5.0, false)
    }
    
    /// Creates a new enemy bullet aimed at a specific direction
    pub fn new_aimed_enemy_bullet(x: f64, y: f64, dx: f64, dy: f64) -> Self {
        let speed = 5.0;
        Self::new(x, y, dx * speed, dy * speed, false)
    }
    
    /// Check if bullet was fired by player
    pub fn is_from_player(&self) -> bool {
        self.from_player
    }
    
    /// Deactivate the bullet (e.g., when it hits something)
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Entity for Bullet {
    fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    
    fn size(&self) -> f64 {
        self.size
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn update(&mut self) {
        if !self.active {
            return;
        }
        
        // Update position
        self.x += self.vx;
        self.y += self.vy;
        
        // Check if bullet is off-screen
        if self.y < -20.0 || self.y > 620.0 || self.x < -20.0 || self.x > 620.0 {
            self.active = false;
        }
    }
} 