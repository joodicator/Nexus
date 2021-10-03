use nxs_interface::{
    self as nxs,
    util::dyn_cast::DynCast,
    root::{LeafModule, RootModule},
    text::TextManager,
};

#[derive(DynCast, LeafModule)]
struct StdTextManager {
    root: &'static dyn RootModule,
}

impl StdTextManager {
    async fn load(root: &'static dyn RootModule) -> nxs::Result<StdTextManager> {
        Ok(StdTextManager {
            root,
        })
    }
}

impl TextManager for StdTextManager {
}
