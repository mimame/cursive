use crate::direction::Direction;
use crate::event::{AnyCb, Event, EventResult};
use crate::rect::Rect;
use crate::vec::Vec2;
use crate::view::{Selector, View};
use crate::Printer;
use std::any::Any;

/// Generic wrapper around a view.
///
/// This trait is a shortcut to implement `View` on a type by forwarding
/// calls to a wrapped view.
///
/// You only need to define `with_view` and `with_view_mut`
/// (the [`wrap_impl!`] macro can help you with that), and you will get
/// the `View` implementation for free.
///
/// You can also override any of the `wrap_*` methods for more specific
/// behaviors (the default implementations simply forwards the calls to the
/// child view).
///
/// [`wrap_impl!`]: ../macro.wrap_impl.html
pub trait ViewWrapper: 'static {
    /// Type that this view wraps.
    type V: View + ?Sized;

    /// Runs a function on the inner view, returning the result.
    ///
    /// Returns `None` if the inner view is unavailable.  This should only
    /// happen with some views if they are already borrowed by another call.
    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R;

    /// Runs a function on the inner view, returning the result.
    ///
    /// Returns `None` if the inner view is unavailable.  This should only
    /// happen with some views if they are already borrowed by another call.
    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R;

    /// Attempts to retrieve the inner view.
    fn into_inner(self) -> Result<Self::V, Self>
    where
        Self: Sized,
        Self::V: Sized,
    {
        Err(self)
    }

    /// Wraps the `draw` method.
    fn wrap_draw(&self, printer: &Printer<'_, '_>) {
        self.with_view(|v| v.draw(printer));
    }

    /// Wraps the `required_size` method.
    fn wrap_required_size(&mut self, req: Vec2) -> Vec2 {
        self.with_view_mut(|v| v.required_size(req))
            .unwrap_or_else(Vec2::zero)
    }

    /// Wraps the `on_event` method.
    fn wrap_on_event(&mut self, ch: Event) -> EventResult {
        self.with_view_mut(|v| v.on_event(ch))
            .unwrap_or(EventResult::Ignored)
    }

    /// Wraps the `layout` method.
    fn wrap_layout(&mut self, size: Vec2) {
        self.with_view_mut(|v| v.layout(size));
    }

    /// Wraps the `take_focus` method.
    fn wrap_take_focus(&mut self, source: Direction) -> bool {
        self.with_view_mut(|v| v.take_focus(source))
            .unwrap_or(false)
    }

    /// Wraps the `find` method.
    fn wrap_call_on_any<'a>(
        &mut self,
        selector: &Selector<'_>,
        callback: AnyCb<'a>,
    ) {
        self.with_view_mut(|v| v.call_on_any(selector, callback));
    }

    /// Wraps the `focus_view` method.
    fn wrap_focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ()> {
        self.with_view_mut(|v| v.focus_view(selector))
            .unwrap_or(Err(()))
    }

    /// Wraps the `needs_relayout` method.
    fn wrap_needs_relayout(&self) -> bool {
        self.with_view(View::needs_relayout).unwrap_or(true)
    }

    /// Wraps the `important_area` method.
    fn wrap_important_area(&self, size: Vec2) -> Rect {
        self.with_view(|v| v.important_area(size))
            .unwrap_or_else(|| Rect::from((0, 0)))
    }
}

// The main point of implementing ViewWrapper is to have View for free.
impl<T: ViewWrapper> View for T {
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.wrap_draw(printer);
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        self.wrap_required_size(req)
    }

    fn on_event(&mut self, ch: Event) -> EventResult {
        self.wrap_on_event(ch)
    }

    fn layout(&mut self, size: Vec2) {
        self.wrap_layout(size);
    }

    fn take_focus(&mut self, source: Direction) -> bool {
        self.wrap_take_focus(source)
    }

    fn call_on_any<'a>(
        &mut self,
        selector: &Selector<'_>,
        callback: Box<dyn FnMut(&mut dyn Any) + 'a>,
    ) {
        self.wrap_call_on_any(selector, callback)
    }

    fn needs_relayout(&self) -> bool {
        self.wrap_needs_relayout()
    }

    fn focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ()> {
        self.wrap_focus_view(selector)
    }

    fn important_area(&self, size: Vec2) -> Rect {
        self.wrap_important_area(size)
    }
}

/// Convenient macro to implement the [`ViewWrapper`] trait.
///
/// It defines the `with_view` and `with_view_mut` implementations,
/// as well as the `type V` declaration.
///
/// [`ViewWrapper`]: view/trait.ViewWrapper.html
///
/// # Examples
///
/// ```rust
/// # use cursive::view::{View,ViewWrapper};
/// struct FooView<T: View> {
///     view: T,
/// }
///
/// impl <T: View> ViewWrapper for FooView<T> {
///     cursive::wrap_impl!(self.view: T);
/// }
/// # fn main() { }
/// ```
#[macro_export]
macro_rules! wrap_impl {
    (self.$v:ident: $t:ty) => {
        type V = $t;

        fn with_view<F, R>(&self, f: F) -> Option<R>
            where F: FnOnce(&Self::V) -> R
        {
            Some(f(&self.$v))
        }

        fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
            where F: FnOnce(&mut Self::V) -> R
        {
            Some(f(&mut self.$v))
        }

        fn into_inner(self) -> Result<Self::V, Self> where Self::V: Sized {
            Ok(self.$v)
        }
    };
}

/// Convenient macro to implement the getters for inner [`View`] in
/// [`ViewWrapper`].
///
/// It defines the `get_inner` and `get_inner_mut` implementations.
///
/// [`ViewWrapper`]: view/trait.ViewWrapper.html
/// [`View`]: view/trait.View.html
///
/// # Examples
///
/// ```rust
/// # use cursive::view::{View, ViewWrapper};
/// struct FooView<T: View> {
///     view: T,
/// }
///
/// impl<T: View> FooView<T> {
///     cursive::inner_getters!(self.view: T);
/// }
///
/// impl <T: View> ViewWrapper for FooView<T> {
///     cursive::wrap_impl!(self.view: T);
/// }
/// # fn main() { }
/// ```
#[macro_export]
macro_rules! inner_getters {
    (self.$v:ident: $t:ty) => {
        /// Gets access to the inner view.
        pub fn get_inner(&self) -> &$t {
            &self.$v
        }
        /// Gets mutable access to the inner view.
        pub fn get_inner_mut(&mut self) -> &mut $t {
            &mut self.$v
        }
    }
}
