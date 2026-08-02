---
name: Ratatui Best Practices
description: Guidelines for building TUIs with Ratatui
---

# Ratatui TUI Patterns

- **Separation of Concerns:** Keep the application state (`app.rs`) completely decoupled from the rendering logic (`ui.rs`).
- **Immediate Mode UI:** Ratatui redraws the whole screen every tick/event. Do not store UI widget objects in the state. Store pure data in `App` and instantiate widgets inside `ui.rs`.
- **Layout:** Use `Layout::default().direction(...).constraints(...)` to structure the screen. Use percentages or fixed lengths. 
- **Popups:** To draw a popup (modal), calculate a centered `Rect` and use `Clear` widget first before drawing your `Block`/`Paragraph` on top of it.
- **Stateful Widgets:** When using `List` or `Table`, you must use `render_stateful_widget` and pass a mutable reference to its `ListState` / `TableState` (which should be stored in `App`).
