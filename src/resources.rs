use bevy::{prelude::*, utils::HashSet};
use std::{hash::Hash, ops::Deref};

#[derive(Resource, Clone)]
pub struct TextStyles {
    regular: Handle<Font>,
    bold: Handle<Font>,
    italic: Handle<Font>,
    italic_bold: Handle<Font>,
    size_multiplier: f32,
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
            font_size: self.size_multiplier * *font_size,
            color,
        }
    }
}

impl FromWorld for TextStyles {
    fn from_world(world: &mut World) -> Self {
        let window = world.query::<&Window>().single(world);
        let asset_server = world.resource::<AssetServer>();
        Self {
            regular: asset_server.load("fonts/OpenSans/OpenSans-Regular.ttf"),
            bold: asset_server.load("fonts/OpenSans/OpenSans-Bold.ttf"),
            italic: asset_server.load("fonts/OpenSans/OpenSans-Italic.ttf"),
            italic_bold: asset_server.load("fonts/OpenSans/OpenSans-BoldItalic.ttf"),
            size_multiplier: 1.0 / window.scale_factor(),
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

#[derive(Debug, Clone, Resource, Reflect)]
pub struct Inputs<T: Copy + Eq + Hash + Send + Sync + 'static> {
    /// A collection of every button that is currently being pressed.
    pressed: HashSet<T>,
    /// A collection of every button that has just been pressed.
    just_pressed: HashSet<T>,
    /// A collection of every button that has just been released.
    just_released: HashSet<T>,
}

impl<T: Copy + Eq + Hash + Send + Sync + 'static> Default for Inputs<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T> Inputs<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    pub fn iter_just_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_pressed.iter()
    }

    pub fn iter_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.pressed.iter()
    }

    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
