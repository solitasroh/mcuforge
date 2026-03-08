use iocraft::prelude::*;

/// Render an element to stdout with ANSI color support
pub fn render<T: Component>(mut element: Element<'_, T>) {
    // iocraft's print() handles terminal detection and sizing
    element.print();
}
