use super::loadable::Loadable;
use bevy::{
    asset::{AssetLoader, AssetPath, AsyncReadExt},
    prelude::*,
    text::FontLoaderError,
};
use bevy_egui::egui;
use std::ops::Deref;

#[derive(Asset, TypePath, Deref, Clone)]
pub struct FontData(egui::FontData);

pub struct FontDataLoader;

impl AssetLoader for FontDataLoader {
    type Asset = FontData;

    type Settings = ();

    type Error = FontLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(FontData(egui::FontData::from_owned(bytes)))
        })
    }
}

#[derive(Resource, Clone)]
pub struct TextStyles {
    regular: Handle<FontData>,
    bold: Handle<FontData>,
    italic: Handle<FontData>,
    italic_bold: Handle<FontData>,
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
        asset_server.register_loader(FontDataLoader);

        Self {
            regular: asset_server.load(FontType::Regular),
            bold: asset_server.load(FontType::Bold),
            italic: asset_server.load(FontType::Italic),
            italic_bold: asset_server.load(FontType::ItalicBold),
        }
    }
}

pub trait TextStylesLoader {
    fn load_text_styles(
        &mut self,
        text_styles: &TextStyles,
        font_data: &mut Assets<FontData>,
    ) -> &mut Self;
}

impl TextStylesLoader for egui::Context {
    fn load_text_styles(
        &mut self,
        text_styles: &TextStyles,
        font_data: &mut Assets<FontData>,
    ) -> &mut Self {
        let mut font_definitions = egui::FontDefinitions::empty();
        let mut push_name = |n: String| {
            font_definitions
                .families
                .insert(egui::FontFamily::Name(n.clone().into()), vec![n.clone()]);
            font_definitions
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push(n);
        };

        let name = FontType::Regular.as_ref().to_string();
        font_definitions.font_data.insert(
            name.clone(),
            font_data.get(&text_styles.regular).unwrap().0.clone(),
        );
        push_name(name);

        let name = FontType::Bold.as_ref().to_string();
        font_definitions.font_data.insert(
            name.clone(),
            font_data.get(&text_styles.bold).unwrap().0.clone(),
        );
        push_name(name);

        let name = FontType::Italic.as_ref().to_string();
        font_definitions.font_data.insert(
            name.clone(),
            font_data.get(&text_styles.italic).unwrap().0.clone(),
        );
        push_name(name);

        let name = FontType::ItalicBold.as_ref().to_string();
        font_definitions.font_data.insert(
            name.clone(),
            font_data.get(&text_styles.italic_bold).unwrap().0.clone(),
        );
        push_name(name);

        self.set_fonts(font_definitions);

        self
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum FontType {
    Regular,
    Bold,
    Italic,
    ItalicBold,
}

impl AsRef<str> for FontType {
    fn as_ref(&self) -> &str {
        match self {
            FontType::Regular => "fonts/OpenSans/OpenSans-Regular.ttf",
            FontType::Bold => "fonts/OpenSans/OpenSans-Bold.ttf",
            FontType::Italic => "fonts/OpenSans/OpenSans-Italic.ttf",
            FontType::ItalicBold => "fonts/OpenSans/OpenSans-BoldItalic.ttf",
        }
    }
}

impl<'a> Into<AssetPath<'a>> for FontType {
    fn into(self) -> AssetPath<'a> {
        self.as_ref().to_string().into()
    }
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
