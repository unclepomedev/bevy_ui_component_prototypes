use bevy::prelude::*;

/// The windows from the bottom to the top.
/// The place of a window gives its `GlobalZIndex`: `z_base + z_step * place`, counting from the bottom.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WindowStack {
    order: Vec<Entity>,
    z_base: i32,
    z_step: i32,
    maximized_z: Option<i32>,
}

impl WindowStack {
    pub(crate) fn new(z_base: i32, z_step: i32, maximized_z: Option<i32>) -> Self {
        Self {
            order: Vec::new(),
            z_base,
            z_step,
            maximized_z,
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

    /// The `GlobalZIndex` of `window` from its place in the stack. A maximized window may have another one:
    /// see `WindowDefaultsPlugin::maximized_z`.
    pub fn layer(&self, window: Entity) -> Option<i32> {
        let place = self.order.iter().position(|w| *w == window)?;
        Some(self.layer_at(place))
    }

    /// The layer of the lowest maximized window, if they have one of their own.
    pub fn maximized_z(&self) -> Option<i32> {
        self.maximized_z
    }

    /// The layer of the `rank`th maximized window from the bottom, from 0.
    pub(crate) fn maximized_layer(&self, rank: usize) -> Option<i32> {
        let offset = i32::try_from(rank)
            .unwrap_or(i32::MAX)
            .saturating_mul(self.z_step);
        self.maximized_z.map(|z| z.saturating_add(offset))
    }

    /// Each window with its layer, from the bottom to the top; those that are `maximized` get the maximized ones.
    pub(crate) fn layers_with<'a>(
        &'a self,
        maximized: impl Fn(Entity) -> bool + 'a,
    ) -> impl Iterator<Item = (Entity, i32)> + 'a {
        let mut rank = 0;
        self.layers().map(move |(window, layer)| {
            if !maximized(window) {
                return (window, layer);
            }
            rank += 1;
            (window, self.maximized_layer(rank - 1).unwrap_or(layer))
        })
    }

    /// Each window with the layer of its place, from the bottom to the top.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entities<const N: usize>() -> [Entity; N] {
        let mut world = World::new();
        std::array::from_fn(|_| world.spawn_empty().id())
    }

    fn stack_of(windows: &[Entity]) -> WindowStack {
        let mut stack = WindowStack::new(10, 10, None);
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
    fn the_maximized_layers_start_at_their_own_base_and_keep_the_step() {
        let stack = WindowStack::new(10, 10, Some(500));
        assert_eq!(stack.maximized_layer(0), Some(500));
        assert_eq!(stack.maximized_layer(2), Some(520));
    }

    #[test]
    fn the_maximized_windows_take_the_maximized_layers_in_the_order_of_the_stack() {
        let [a, b, c] = entities();
        let mut stack = WindowStack::new(10, 10, Some(500));
        for window in [a, b, c] {
            stack.push(window);
        }
        let layers: Vec<_> = stack.layers_with(|window| window != b).collect();
        assert_eq!(layers, [(a, 500), (b, 20), (c, 510)]);
    }

    #[test]
    fn without_a_maximized_base_there_is_no_maximized_layer() {
        assert_eq!(WindowStack::new(10, 10, None).maximized_layer(0), None);
    }

    #[test]
    fn the_layers_do_not_overflow() {
        let [a, b] = entities();
        let mut stack = WindowStack::new(i32::MAX - 5, 10, None);
        stack.push(a);
        stack.push(b);
        assert_eq!(stack.layer(b), Some(i32::MAX));
    }
}
