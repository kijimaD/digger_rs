use rltk::rex::XpFile;

rltk::embedded_resource!(LIGHT_HOUSE, "../resources/LightHouse_80x60.xp");
rltk::embedded_resource!(FOREST, "../resources/Forest_80x60.xp");

pub struct RexAssets {
    pub menu: XpFile,
    pub forest: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(LIGHT_HOUSE, "../resources/LightHouse_80x60.xp");
        rltk::link_resource!(FOREST, "../resources/Forest_80x60.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/LightHouse_80x60.xp").unwrap(),
            forest: XpFile::from_resource("../resources/Forest_80x60.xp").unwrap(),
        }
    }
}
