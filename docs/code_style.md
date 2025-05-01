# Notes on Code Quality (Non-ECS Version)

Last Updated: 2025-05-01

## 1. Coding Conventions

### 1.1 Rust Style Guidelines

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Refer to the [Rust API Design Guidelines](https://rust-lang.github.io/api-guidelines/)

### 1.2 Naming Conventions

| Element         | Convention       | Example              |
|----------------|------------------|----------------------|
| Modules         | `snake_case`     | `game_state`         |
| Structs / Enums | `PascalCase`     | `GameState`          |
| Functions       | `snake_case`     | `update_state`       |
| Constants       | `SCREAMING_SNAKE_CASE` | `MAX_PLAYERS` |
| Macros          | `snake_case!`    | `println!`           |

### 1.3 Documentation Comments

Add appropriate documentation to all public APIs:

```rust
/// Represents the current game state.
///
/// Holds and updates the internal state of the game.
pub struct GameState {
    // fields
}

impl GameState {
    /// Creates a new game state.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas element to render the game on
    ///
    /// # Returns
    ///
    /// A new instance of `GameState`, or an initialization error
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // implementation
    }
}
```

## 2. Error Handling

### 2.1 Use `Result` Instead of `panic!`

- Avoid using `panic!` in production code.
- Always return `Result<T, E>` for fallible operations.
- Use `?` operator to propagate errors when appropriate.
- For user-facing features, convert internal errors into descriptive messages.

```rust
fn parse_config(data: &str) -> Result<Config, JsValue> {
    let config: Config = serde_json::from_str(data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {e}")))?;
    Ok(config)
}
```

---

### 2.2 Provide Meaningful Error Messages

- Error messages should be actionable and specific.
- Avoid vague messages like `"Something went wrong"`.
- When returning `JsValue`, include context for debugging.

```rust
Err(JsValue::from_str("WebSocket connection failed: invalid URL"))
```

---

### 2.3 Use `Option` Safely

- Always handle `None` cases explicitly.
- Prefer `unwrap_or`, `map`, and `and_then` over `unwrap`.

```rust
// Good
let speed = config.player_speed.unwrap_or(DEFAULT_SPEED);

// Avoid this
let speed = config.player_speed.unwrap(); // May panic
```

---

### 2.4 Define Custom Error Types (if needed)

- When the error domain grows, define your own `enum` error type.
- Implement `Display`, `Debug`, and `From` traits if converting from other error types.

```rust
#[derive(Debug)]
pub enum GameError {
    ConfigError(String),
    NetworkError(String),
    Unexpected(String),
}

impl From<serde_json::Error> for GameError {
    fn from(err: serde_json::Error) -> Self {
        GameError::ConfigError(err.to_string())
    }
}
```

---

### 2.5 Wrap JavaScript Errors Properly

- Always convert JS errors into `JsValue` and annotate them with clear context.
- For external API calls, use `map_err` to wrap errors.

```rust
let value = js_sys::Reflect::get(&object, &JsValue::from_str("field"))
    .map_err(|e| JsValue::from_str(&format!("JS field access error: {e:?}")))?;
```

---

### 2.6 Avoid Silent Failures

- Never ignore `Result` without at least using `let _ = ...`.
- Document when intentional error-ignoring is done (e.g., in UI rendering fallback).

```rust
// OK: intentional fallback, clearly documented
let _ = self.context.translate(x, y); // Failing here is non-critical
```

---

### 2.7 Propagate Errors Cleanly

- Use `Result<(), JsValue>` in WASM-facing functions.
- Provide early returns on error to reduce nesting.

```rust
pub fn render(&self) -> Result<(), JsValue> {
    if self.canvas.width() == 0 {
        return Err(JsValue::from_str("Canvas width is zero"));
    }

    self.draw_background()?;
    self.draw_entities()?;
    Ok(())
}
```
## 3. Testing


OKあさべっち、行くで！🔥  
**「3. Testing」セクション（Non-ECS版）英訳済みバージョン**を以下に示すね：

---

## 🧪 3. Testing

### 3.1 Unit Testing

- Place unit tests inside the same module using `#[cfg(test)]`.
- Use `wasm-bindgen-test` for browser-compatible tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_default_score() {
        let game = GameState::new_mock();
        assert_eq!(game.score, 0);
    }
}
```

- Use `#[should_panic]` to assert failure cases when appropriate.

```rust
#[test]
#[should_panic(expected = "Invalid level")]
fn test_invalid_level() {
    GameState::with_level(-1);
}
```

---

### 3.2 Mocking Web APIs

- For DOM and Web APIs, use `web_sys` stubs or mocks in test environments.
- Abstract over `HtmlCanvasElement`, `CanvasRenderingContext2d`, etc., with interfaces or wrappers.
- Use `#[wasm_bindgen_test]` to verify rendering logic.

```rust
#[wasm_bindgen_test]
fn test_canvas_context_initialization() {
    let canvas = create_mock_canvas();
    let ctx = get_canvas_context(&canvas).unwrap();
    assert_eq!(ctx.global_alpha(), 1.0);
}
```

---

### 3.3 E2E Testing Strategy

- Use browser automation tools like **Playwright** or **Puppeteer** for full end-to-end testing.
- Write integration scripts in JS/TS to drive the WASM UI from outside.
- Test the following:
  - User input via mouse / keyboard
  - WebSocket communication
  - UI state changes
  - Error recovery paths

```js
// Example (Puppeteer)
await page.click('#cell-4-5');
const text = await page.$eval('#score', el => el.textContent);
expect(text).toBe('1');
```

- Use `cargo test` for logic-heavy modules.
- Use `wasm-pack test --chrome` or similar for browser integration.

## 4. Performance Optimization

### 4.1 Memory Management

- Avoid unnecessary heap allocations inside hot paths (like rendering loops).
- Reuse buffers and vectors whenever possible.
- Be aware of the cost of cloning large structures.

```rust
// Avoid reallocating in a loop
let mut buffer = Vec::with_capacity(1000);
for i in 0..1000 {
    buffer.push(i);
}
```

- For dynamic data structures, prefer `VecDeque`, `SmallVec`, or pooling if performance-critical.

---

### 4.2 Batch Processing and Object Pooling

- Group similar operations into batches to minimize API overhead and loop conditions.

```rust
// Instead of rendering one by one:
for sprite in &sprites {
    sprite.draw(&ctx)?;
}

// Better: group by texture, layer, etc., and draw in batches
```

- Reuse frequently created objects with pooling:

```rust
let mut pool = object_pool.borrow_mut();
let obj = pool.get_or_create(|| ExpensiveObject::new());
```

---

### 4.3 Canvas Rendering Optimization

- Use **dirty flags** to redraw only when needed.
- Avoid full canvas clearing unless required.
- Minimize style changes (e.g. `fill_style`, `font`, `transform`) between draw calls.

```rust
// Only redraw when state changes
if self.needs_redraw {
    self.render(ctx)?;
    self.needs_redraw = false;
}
```

- Use `save()` / `restore()` blocks to localize canvas state changes.

```rust
ctx.save();
ctx.set_fill_style(&JsValue::from_str("black"));
ctx.fill_text("Score: 10", 10.0, 20.0)?;
ctx.restore();
```

---

### 4.4 Memory Profiling Tools

- Use browser DevTools to track:
  - WASM heap usage
  - JS interop frequency
  - Canvas reflow / repaint cost

- For native profiling:
  - Use `cargo-flamegraph` on Rust-side
  - Use `wasm-bindgen`’s debug builds + DevTools heap snapshot

## 5. Security

### 5.1 Input Validation

- Always validate **user input** on both the frontend and backend (even if WASM is client-side only).
- Sanitize string input before using it in DOM APIs or rendering.

```rust
// Never trust direct user input
fn handle_username_input(input: &str) -> Result<(), JsValue> {
    if input.len() > 20 {
        return Err(JsValue::from_str("Username too long"));
    }
    // Additional sanitization if used in HTML
    Ok(())
}
```

- Use enums or validation functions to constrain input to expected values.

---

### 5.2 WebSocket Message Verification

- All incoming WebSocket messages must be:
  - **Schema-validated** (e.g., using `serde_json::from_str`)
  - **Type-checked**
  - **Boundary-checked**

```rust
#[derive(Deserialize)]
struct ClientMessage {
    kind: String,
    payload: Value,
}

fn handle_message(raw: &str) -> Result<(), JsValue> {
    let msg: ClientMessage = serde_json::from_str(raw)
        .map_err(|_| JsValue::from_str("Malformed message"))?;

    match msg.kind.as_str() {
        "join" => handle_join(msg.payload),
        "action" => handle_action(msg.payload),
        _ => Err(JsValue::from_str("Unknown message type")),
    }
}
```

- Avoid unsafe casting or unchecked deserialization.
- Never trust the message origin — check player ID or session token manually if needed.

## 6. Code Review Criteria

### 6.1 Functional Correctness

- Does the code **achieve its intended purpose**?
- Are all edge cases and failure scenarios **handled properly**?
- Are the inputs and outputs **clearly defined**?

### 6.2 Coding Standard Compliance

- Does the code follow **Rust style guidelines** and **project-specific conventions**?
- Are naming conventions and formatting consistent?
- Is documentation provided for all public items?

### 6.3 Test Coverage

- Are **unit tests** provided for new modules or functions?
- Are **error paths** and **edge cases** tested?
- Is **E2E behavior** covered where applicable?

### 6.4 Performance Considerations

- Are there any **unnecessary allocations** or **hot-loop inefficiencies**?
- Is **Canvas rendering optimized** (e.g., dirty flag, minimal repainting)?
- Are **browser/JS interop calls minimized**?

### 6.5 Security Checks

- Is all **user input properly validated**?
- Are **WebSocket messages sanitized and type-checked**?
- Are there any potential **DoS vectors** (e.g., infinite loops, unbounded memory growth)?

## 7. Refactoring Best Practices

### 7.1 Small Incremental Changes  
- Break changes into **small, reviewable commits**.  
- Avoid large diffs that mix multiple concerns.  
- Each commit should ideally be **self-contained** and **testable**.

### 7.2 Behavior Preservation  
- Confirm that behavior **does not change** after refactoring.  
- Use tests or manual validation to **verify equivalence**.  
- If changes affect behavior, document the **reason and impact** clearly.

### 7.3 Separating Refactor from Feature Development  
- **Do not mix** refactoring with new feature additions.  
- Submit **refactoring as its own PR** or commit group.  
- This improves clarity and simplifies reviews and rollbacks.

## 8. Rust-Specific Best Practices

### 8.1 Ownership and Borrowing  
- Ensure **consistent parameter types** in function signatures (value vs reference).  
- Avoid mixing **owned values and references** unnecessarily.  
- Be cautious with `*` dereferencing; **document the expected types**.  
- Use **clear and descriptive documentation** to indicate expected ownership behavior.

### 8.2 Dealing with Unused Variables  
- Use an **underscore prefix (`_`)** for intentionally unused variables.  
- Use `_` alone for totally ignored values (e.g. loop counters).  
- Maintain **signature consistency** even when some parameters are unused.

```rust
fn update_state(&mut self, _delta_time: f32) {
    // _delta_time is intentionally unused
}
```

### 8.3 Explicit Type Annotations  
- Use type annotations when the **compiler’s inference might be unclear**.  
- Especially useful for **math operations**, **method chaining**, and **generic types**.  
- Improves **readability** and **compiler diagnostics**.

```rust
let score: u32 = player.get_score();
```

### 8.4 `Result` Handling  
- Always handle `Result` values explicitly:  
  - Use `?` to propagate errors  
  - Use `let _ =` to **suppress warnings** when ignoring  
- Add **comments** when intentionally ignoring errors

```rust
let _ = self.context.translate(x, y); // Not critical if this fails
```

### 8.5 Concurrency Patterns  
- Clearly define **resource access rules** across threads/systems  
- Use `RefCell`, `Rc`, `Arc`, and `Mutex` appropriately  
- Avoid shared state when possible; favor **message passing** or **event queues**

了解！それでは「**9. WebAssembly + Canvas Integration（WebAssembly＋Canvas統合）**」の英語版セクションをお届けします！

---

## 9. WebAssembly + Canvas Integration

### 9.1 Interfacing with JavaScript  
- Use `wasm_bindgen` to expose Rust functions to JS.  
- Prefer **typed bindings** over raw JS interop for safety.  
- Validate DOM access (`document`, `canvas`) to avoid `null` dereferencing.

```rust
let canvas: HtmlCanvasElement = document
    .get_element_by_id("game-canvas")?
    .dyn_into()?;
```

### 9.2 Canvas API Usage  
- Use `web_sys::CanvasRenderingContext2d` for 2D rendering.  
- Always check for and **handle `Result<T, JsValue>`** in draw operations.  
- Use `save()` / `restore()` to isolate transformation states.

```rust
let _ = self.context.save();
let _ = self.context.translate(x, y);
let _ = self.context.restore();
```

### 9.3 Error Handling in Canvas Ops  
- Canvas methods like `translate()`, `scale()`, and `rotate()` return `Result`; handle it properly.  
- Use `?` when errors should bubble up; use `let _ =` with comments if ignoring is intentional.  
- Avoid silent failures—**log when skipping errors intentionally**.

### 9.4 Performance Tips for Canvas  
- Minimize draw calls by **only updating dirty regions**.  
- Use **requestAnimationFrame** from JS for synced rendering.  
- Avoid layout thrashing: **batch style changes** and **avoid DOM reads during render**.  
- Keep offscreen rendering buffers when possible.

### 9.5 DOM Interaction Minimization  
- Avoid frequent DOM access from Rust—**cache nodes or use state diffing**.  
- Defer UI updates to JS if possible, using `Closure` and `setTimeout`/`requestAnimationFrame`.  
- Keep communication between Rust and JS **minimal, typed, and predictable**.