use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/LightHouse_80x60.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/LightHouse_80x60.xp");

        RexAssets { menu: XpFile::from_resource("../resources/LightHouse_80x60.xp").unwrap() }
    }
}
