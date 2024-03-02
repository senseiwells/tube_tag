use iced::{Element, Event, event, Length, mouse, Point, Rectangle, Size, Vector};
use iced::advanced::{Clipboard, Layout, Overlay, renderer, Shell, Widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree::{State, Tag};
use iced::event::Status;
use iced::mouse::{Cursor, Interaction};

// I never want to touch this file again - sensei :)

/// Widget for rendering a purely visual overlay over an element.
/// This will conserve all functionality and events for the underlying element.
/// This also passes in the underlying element's state into the
/// overlaying element's draw call.
pub struct RenderOverlay<'a, Message, Theme, Renderer> {
    underlay: Element<'a, Message, Theme, Renderer>,
    overlay: Element<'a, Message, Theme, Renderer>
}

impl<'a, Message, Theme, Renderer> RenderOverlay<'a, Message, Theme, Renderer> {
    pub fn new(
        underlay: impl Into<Element<'a, Message, Theme, Renderer>>,
        overlay: impl Into<Element<'a, Message, Theme, Renderer>>
    ) -> RenderOverlay<'a, Message, Theme, Renderer> {
        return Self {
            underlay: underlay.into(),
            overlay: overlay.into()
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for RenderOverlay<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer
{
    fn size(&self) -> Size<Length> {
        self.underlay.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.underlay.as_widget().layout(tree, renderer, limits)
    }

    fn draw(
        &self, tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle
    ) {
        // We can only draw the underlay here
        // Rendering the overlay here does not guarantee that it will
        // actually be on top despite calling the draw method after
        self.underlay.as_widget().draw(
            &tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport
        );
    }

    fn tag(&self) -> Tag {
        self.underlay.as_widget().tag()
    }

    fn state(&self) -> State {
        self.underlay.as_widget().state()
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.underlay.as_widget_mut().on_event(
            state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        self.underlay.as_widget().mouse_interaction(
            state,
            layout,
            cursor,
            viewport,
            renderer
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        translation: Vector
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        Some(iced::advanced::overlay::Element::new(Box::new(RenderedOverlayWithState {
            underlay: &mut self.underlay,
            overlay: &mut self.overlay,
            state,
            position: layout.position() + translation,
            bounds: layout.bounds()
        })))
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer> From<RenderOverlay<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a
{
    fn from(value: RenderOverlay<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

struct RenderedOverlayWithState<'a, 'b, Message, Theme, Renderer> {
    underlay: &'b mut Element<'a, Message, Theme, Renderer>,
    overlay: &'b mut Element<'a, Message, Theme, Renderer>,
    state: &'b mut Tree,
    position: Point,
    bounds: Rectangle
}

impl<'a, 'b, Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for RenderedOverlayWithState<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer
{
    fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> Node {
        let limits = Limits::new(Size::ZERO, self.bounds.size())
            .width(Length::Fill)
            .height(Length::Fill);
        let node = self
            .underlay
            .as_widget()
            .layout(self.state, renderer, &limits);
        node.move_to(self.position)
    }

    fn draw(
        &self, renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor
    ) {
        self.overlay.as_widget().draw(
            self.state,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds()
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>
    ) -> Status {
        // We have to do this check otherwise we will consume events for other widgets
        if cursor.is_over(layout.bounds()) {
            return self.underlay.as_widget_mut().on_event(self.state, event, layout, cursor, renderer, clipboard, shell, &layout.bounds())
        }
        Status::Ignored
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer
    ) -> Interaction {
        // We have to do this check otherwise we will consume events for other widgets
        if cursor.is_over(viewport.clone()) {
            return self.underlay.as_widget().mouse_interaction(self.state, layout, cursor, viewport, renderer)
        }
        Interaction::Idle
    }
}