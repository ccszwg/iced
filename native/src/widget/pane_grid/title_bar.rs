use crate::container;
use crate::layout;
use crate::pane_grid;
use crate::{Clipboard, Element, Event, Layout, Point, Size};

pub struct TitleBar<'a, Message, Renderer: container::Renderer> {
    title: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: u16,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: container::Renderer,
{
    pub fn new(title: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title: title.into(),
            controls: None,
            padding: 0,
            style: Renderer::Style::default(),
        }
    }

    pub fn controls(
        mut self,
        controls: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.controls = Some(controls.into());
        self
    }

    /// Sets the padding of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the style of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        show_controls: bool,
    ) -> Renderer::Output {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        if let Some(controls) = &self.controls {
            let mut children = padded.children();
            let title_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            renderer.draw_title_bar(
                defaults,
                layout.bounds(),
                &self.style,
                (&self.title, title_layout),
                if show_controls {
                    Some((controls, controls_layout))
                } else {
                    None
                },
                cursor_position,
            )
        } else {
            renderer.draw_title_bar(
                defaults,
                layout.bounds(),
                &self.style,
                (&self.title, padded),
                None,
                cursor_position,
            )
        }
    }

    pub fn is_over_draggable(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> bool {
        if layout.bounds().contains(cursor_position) {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            if self.controls.is_some() {
                let mut children = padded.children();
                let _ = children.next().unwrap();
                let controls_layout = children.next().unwrap();

                !controls_layout.bounds().contains(cursor_position)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits.pad(padding);

        let node = if let Some(controls) = &self.controls {
            let max_size = limits.max();

            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            let mut title_layout = self.title.layout(
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(space_before_controls, max_size.height),
                ),
            );

            title_layout.move_to(Point::new(padding, padding));
            controls_layout
                .move_to(Point::new(space_before_controls + padding, padding));

            let title_size = title_layout.size();
            let height = title_size.height.max(controls_size.height);

            layout::Node::with_children(
                Size::new(max_size.width, height),
                vec![title_layout, controls_layout],
            )
        } else {
            self.title.layout(renderer, &limits)
        };

        layout::Node::with_children(node.size().pad(padding), vec![node])
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        if let Some(controls) = &mut self.controls {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            let mut children = padded.children();
            let _ = children.next();
            let controls_layout = children.next().unwrap();

            controls.on_event(
                event,
                controls_layout,
                cursor_position,
                messages,
                renderer,
                clipboard,
            );
        }
    }
}
