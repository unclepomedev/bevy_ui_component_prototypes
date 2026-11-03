use bevy::prelude::*;

/// The windows from the bottom to the top.
/// The place of a window gives its `GlobalZIndex`: `z_base + z_step * place`, counting from the bottom.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WindowStack {
    order: Vec<Entity>,
    z_base: i32,
    z_step: i32,
}

impl WindowStack {
    pub(crate) fn new(z_base: i32, z_step: i32) -> Self {
        Self {
            order: Vec::new(),
            z_base,
            z_step,
        }
    }

    /// From the bottom to the top.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Entity> + '_ {
        self.order.iter().copied()
    }

    pub fn top(&self) -> Option<Entity> {
        self.order.last().copied()
    }

    pub fn contains(&self, window: Entity) -> bool {
        self.order.contains(&window)
    }

    /// The `GlobalZIndex` of `window`, if it is in the stack.
    pub fn layer(&self, window: Entity) -> Option<i32> {
        let place = self.order.iter().position(|w| *w == window)?;
        Some(self.layer_at(place))
    }

    /// Each window with its `GlobalZIndex`, from the bottom to the top.
    pub(crate) fn layers(&self) -> impl Iterator<Item = (Entity, i32)> + '_ {
        self.order
            .iter()
            .enumerate()
            .map(|(place, window)| (*window, self.layer_at(place)))
    }

    fn layer_at(&self, place: usize) -> i32 {
        let offset = i32::try_from(place)
            .unwrap_or(i32::MAX)
            .saturating_mul(self.z_step);
        self.z_base.saturating_add(offset)
    }

    /// Puts `window` on the top. Nothing happens if it is in the stack already.
    pub(crate) fn push(&mut self, window: Entity) -> bool {
        if self.contains(window) {
            return false;
        }
        self.order.push(window);
        true
    }

    pub(crate) fn remove(&mut self, window: Entity) -> bool {
        let Some(place) = self.order.iter().position(|w| *w == window) else {
            return false;
        };
        self.order.remove(place);
        true
    }

    /// Moves `window` to the top. Nothing happens if it is not in the stack or is on the top.
    pub(crate) fn raise(&mut self, window: Entity) -> bool {
        if self.top() == Some(window) || !self.remove(window) {
            return false;
        }
        self.order.push(window);
        true
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn entities<const N: usize>() -> [Entity; N] {
        let mut world = World::new();
        std::array::from_fn(|_| world.spawn_empty().id())
    }

    fn stack_of(windows: &[Entity]) -> WindowStack {
        let mut stack = WindowStack::new(10, 10);
        for window in windows {
            stack.push(*window);
        }
        stack
    }

    #[test]
    fn a_window_pushed_later_is_higher() {
        let [a, b, c] = entities();
        let stack = stack_of(&[a, b, c]);
        assert_eq!(stack.iter().collect::<Vec<_>>(), [a, b, c]);
        assert_eq!(stack.top(), Some(c));
    }

    #[test]
    fn the_layers_are_the_base_plus_the_step_per_place() {
        let [a, b, c] = entities();
        let stack = stack_of(&[a, b, c]);
        assert_eq!(stack.layer(a), Some(10));
        assert_eq!(stack.layer(b), Some(20));
        assert_eq!(stack.layer(c), Some(30));
        assert_eq!(
            stack.layers().collect::<Vec<_>>(),
            [(a, 10), (b, 20), (c, 30)]
        );
    }

    #[test]
    fn a_window_that_is_not_in_the_stack_has_no_layer() {
        let [a, b] = entities();
        assert_eq!(stack_of(&[a]).layer(b), None);
    }

    #[test]
    fn pushing_a_window_twice_changes_nothing() {
        let [a, b] = entities();
        let mut stack = stack_of(&[a, b]);
        assert!(!stack.push(a));
        assert_eq!(stack.iter().collect::<Vec<_>>(), [a, b]);
    }

    #[test]
    fn raising_moves_the_window_to_the_top() {
        let [a, b, c] = entities();
        let mut stack = stack_of(&[a, b, c]);
        assert!(stack.raise(a));
        assert_eq!(stack.iter().collect::<Vec<_>>(), [b, c, a]);
        assert_eq!(stack.layer(a), Some(30));
        assert_eq!(stack.layer(b), Some(10));
    }

    #[test]
    fn raising_the_top_or_a_stranger_changes_nothing() {
        let [a, b, stranger] = entities();
        let mut stack = stack_of(&[a, b]);
        assert!(!stack.raise(b));
        assert!(!stack.raise(stranger));
        assert_eq!(stack.iter().collect::<Vec<_>>(), [a, b]);
    }

    #[test]
    fn removing_closes_the_gap() {
        let [a, b, c] = entities();
        let mut stack = stack_of(&[a, b, c]);
        assert!(stack.remove(b));
        assert!(!stack.remove(b));
        assert_eq!(stack.layer(c), Some(20));
    }

    #[test]
    fn the_layers_do_not_overflow() {
        let [a, b] = entities();
        let mut stack = WindowStack::new(i32::MAX - 5, 10);
        stack.push(a);
        stack.push(b);
        assert_eq!(stack.layer(b), Some(i32::MAX));
    }
}
