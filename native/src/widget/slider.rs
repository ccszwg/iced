//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::{
    layout, mouse, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::{hash::Hash, ops::RangeInclusive};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// [`Slider`]: struct.Slider.html
///
/// # Example
/// ```
/// # use iced_native::{slider, renderer::Null};
/// #
/// # pub type Slider<'a, Message> = iced_native::Slider<'a, Message, Null>;
/// pub enum Message {
///     SliderChanged(f32),
/// }
///
/// let state = &mut slider::State::new();
/// let value = 50.0;
///
/// Slider::new(state, 0.0..=100.0, value, Message::SliderChanged);
/// ```
///
/// ![Slider drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/slider.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Slider<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    range: RangeInclusive<f32>,
    value: f32,
    on_change: Box<dyn Fn(f32) -> Message>,
    on_release: Option<Message>,
    width: Length,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> Slider<'a, Message, Renderer> {
    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`Slider`]
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///   It receives the new value of the [`Slider`] and must produce a
    ///   `Message`.
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`State`]: struct.State.html
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<f32>,
        value: f32,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(f32) -> Message,
    {
        Slider {
            state,
            value: value.max(*range.start()).min(*range.end()),
            range,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

/// The local state of a [`Slider`].
///
/// [`Slider`]: struct.Slider.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_dragging: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message, Renderer> Widget<'a, Message, Renderer>
    for Slider<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: Clone,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(Length::Units(renderer.height() as u16));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        let mut change = || {
            let bounds = layout.bounds();

            if cursor_position.x <= bounds.x {
                messages.push((self.on_change)(*self.range.start()));
            } else if cursor_position.x >= bounds.x + bounds.width {
                messages.push((self.on_change)(*self.range.end()));
            } else {
                let percent = (cursor_position.x - bounds.x) / bounds.width;
                let value = (self.range.end() - self.range.start()) * percent
                    + self.range.start();

                messages.push((self.on_change)(value));
            }
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if layout.bounds().contains(cursor_position) {
                        change();
                        self.state.is_dragging = true;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if self.state.is_dragging {
                        if let Some(on_release) = self.on_release.clone() {
                            messages.push(on_release);
                        }
                        self.state.is_dragging = false;
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if self.state.is_dragging {
                        change();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.range.clone(),
            self.value,
            self.state.is_dragging,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
    }
}

/// The renderer of a [`Slider`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Slider`] in your user interface.
///
/// [`Slider`]: struct.Slider.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the height of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    fn height(&self) -> u32;

    /// Draws a [`Slider`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Slider`]
    ///   * the local state of the [`Slider`]
    ///   * the range of values of the [`Slider`]
    ///   * the current value of the [`Slider`]
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        range: RangeInclusive<f32>,
        value: f32,
        is_dragging: bool,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Slider<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a + Clone,
{
    fn from(
        slider: Slider<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(slider)
    }
}
