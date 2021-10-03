use nxs_interface::{
    self as nxs,
    util::dyn_cast::DynCast,
    root::{LeafModule, RootModule},
    text::TextManager,
};

#[derive(DynCast, LeafModule)]
struct Commands {
    root: &'static dyn RootModule,
    text: &'static dyn TextManager,
}

impl Commands {
    async fn load(root: &'static dyn RootModule) -> nxs::Result<Commands> {
        Ok(Commands {
            root,
            text: root.import::<dyn TextManager>().await?,
        })
    }
}
