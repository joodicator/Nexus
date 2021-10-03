use std::any::TypeId;
use nxs_interface::{
    util::{DynCast, DynCastExt, DynCastRef},
    util::root::{RootModule, LeafModule},
}
use futures::future::BoxFuture;

struct StdRootModule;

impl RootModule for StdRootModule {
    fn dyn_import(
        &self, as_type: TypeId
    ) -> BoxFuture<nxs::Result<DynCastRef>> {
        Box::pin(async {
            Err("not implemented")
        })
    }
}
