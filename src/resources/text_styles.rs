use super::loadable::Loadable;
use bevy::prelude::*;
use std::ops::Deref;

#[derive(Resource, Clone)]
pub struct TextStyles {
    regular: Handle<Font>,
    bold: Handle<Font>,
    italic: Handle<Font>,
    italic_bold: Handle<Font>,
}

impl TextStyles {
    pub fn get(&self, font_type: FontType, font_size: FontSize, color: Color) -> TextStyle {
        let font = match font_type {
            FontType::Regular => self.regular.clone(),
            FontType::Bold => self.bold.clone(),
            FontType::Italic => self.italic.clone(),
            FontType::ItalicBold => self.italic_bold.clone(),
        };

        TextStyle {
            font,
            font_size: *font_size,
            color,
        }
    }
}

impl Loadable for TextStyles {
    fn loaded(&self, asset_server: &AssetServer) -> bool {
        asset_server.is_loaded_with_dependencies(&self.regular)
            && asset_server.is_loaded_with_dependencies(&self.bold)
            && asset_server.is_loaded_with_dependencies(&self.italic)
            && asset_server.is_loaded_with_dependencies(&self.italic_bold)
    }
}

impl FromWorld for TextStyles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            regular: asset_server.load("fonts/OpenSans/OpenSans-Regular.ttf"),
            bold: asset_server.load("fonts/OpenSans/OpenSans-Bold.ttf"),
            italic: asset_server.load("fonts/OpenSans/OpenSans-Italic.ttf"),
            italic_bold: asset_server.load("fonts/OpenSans/OpenSans-BoldItalic.ttf"),
        }
    }
}

#[allow(dead_code)]
pub enum FontType {
    Regular,
    Bold,
    Italic,
    ItalicBold,
}

#[allow(dead_code)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    XLarge,
}

impl Deref for FontSize {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        match self {
            FontSize::Small => &32.0,
            FontSize::Medium => &48.0,
            FontSize::Large => &60.0,
            FontSize::XLarge => &72.0,
        }
    }
}
