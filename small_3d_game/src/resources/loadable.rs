use bevy::asset::AssetServer;

pub trait Loadable {
    fn loaded(&self, asset_server: &AssetServer) -> bool;
}
