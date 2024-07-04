use crate::video::sprite::Sprite;
use crate::video::tile::Tile;

pub struct Oam {
    pub sprite: Sprite,
    pub tile1: Tile,
    pub tile2: Option<Tile>,
}
