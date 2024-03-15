use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
    pub italic: Handle<Font>,
    pub italic_bold: Handle<Font>,
}

impl FromWorld for Fonts {
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
