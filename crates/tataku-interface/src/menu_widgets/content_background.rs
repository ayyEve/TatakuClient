use crate::prelude::*;
use crate::prelude::ui::*; 


// TODO: rename this
#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("container"))]
pub struct ContentBackground {
    content: Box<dyn Widget>,
    image: ContentImage,

    #[chain] border: Option<Border>,
    #[chain] color: Option<Color>,
    #[chain] shape: Shape,
    #[chain] style: Style,
    node_id: NodeId,
}
impl ContentBackground {
    /// Creates a new [`ContentWithImage`] with the given content
    pub fn new(content: Box<dyn Widget>) -> Self {
        Self {
            content,
            image: ContentImage::None,
            color: None,
            border: None,
            shape: Shape::Square,

            style: Style::DEFAULT,

            node_id: EMPTY_NODE,
        }
    }
    
    /// set this content's background image
    pub fn image(mut self, image: impl Into<ContentImage>) -> Self {
        let mut image = image.into();
        if let ContentImage::Built(i, _) = &mut image {
            i.origin = Vector2::ZERO
        }
        self.image = image;
        self
    }
}

#[async_trait]
impl Widget for ContentBackground {
    fn name(&self) -> Cow<'static, str> { "content_background_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<'_>,
    ) -> TaffyResult<NodeId> {
        let child = self.content.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            self.style.clone(), 
            &[ child ]
        )?;

        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<'_>
    ) {
        self.content.input(event, shell);
    }


    fn draw(
        &self,
        shell: &mut DrawShell<'_>,
    ) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };

        if let Some(mut image) = self.image.get_img() {
            image.pos = bounds.pos;
            image.set_size(bounds.size);

            shell.list.push(image);
        } else if self.color.is_some() || self.border.is_some() {
            let rect = crate::prelude::Rectangle::new_bounds(
                bounds, 
                self.color.unwrap_or(Color::TRANSPARENT_WHITE),
                self.border
            ).shape(self.shape);
            shell.list.push(rect);
        }

        self.content.draw(shell);
    }


    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        self.content.update(shell, actions)
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        self.content.handle_message(message, values, actions).await
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect
    ) {
        self.content.handle_event(event, event_value.clone(), values).await
    }

    async fn reload_skin(&mut self, skin_manager: &mut dyn SkinProvider) {
        match &mut self.image {
            ContentImage::Unbuilt(path) => {
                let img = skin_manager.get_texture_then(&*path, &TextureSource::Skin, SkinUsage::Game, false, |i| {
                    i.origin = Vector2::ZERO;
                }).await;
                
                if let Some(img) = img {
                    self.image = ContentImage::Built(img, Some(path.clone()))
                }
            }
            ContentImage::Built(i, Some(path)) => {
                let img = skin_manager.get_texture_then(&*path, &TextureSource::Skin, SkinUsage::Game, false, |i| {
                    i.origin = Vector2::ZERO;
                }).await;
                
                if let Some(img) = img {
                    *i = img
                } else {
                    self.image = ContentImage::Unbuilt(path.clone())
                }
            }

            _ => {}
        }

        self.content.reload_skin(skin_manager).await
    }
}


pub enum ContentImage {
    None,
    Built(Image, Option<String>),
    Unbuilt(String),
}
impl ContentImage {
    fn get_img(&self) -> Option<Image> {
        let Self::Built(i, _) = self else { return None };
        Some(i.clone())
    }
}

impl<T: Into<ContentImage>> From<Option<T>> for ContentImage {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::None };
        value.into()
    }
}
impl From<String> for ContentImage {
    fn from(value: String) -> Self {
        Self::Unbuilt(value)
    }
}
impl From<Image> for ContentImage {
    fn from(value: Image) -> Self {
        Self::Built(value, None)
    }
}
