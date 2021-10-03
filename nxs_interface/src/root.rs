use std::any::TypeId;

use crate::{self as nxs, util::dyn_cast::{DynCast, DynCastRef}};

use futures::future::BoxFuture;

pub use root_module::RootModule;
pub use leaf_module::LeafModule;

pub mod root_module {
    use super::*;

    pub trait RootModule: DynCast + Sync {
        fn dyn_import(&'static self, as_type: TypeId)
        -> BoxFuture<nxs::Result<DynCastRef<'_>>>;
    }

    const ROOT_MODULE_ERR: &str =
        "The contract of `RootModule` has been violated by an implementation.";

    pub async fn import_from<M: LeafModule + ?Sized>(
        root: &'static (impl RootModule + ?Sized)
    ) -> nxs::Result<&M> {
        let dyn_ref: DynCastRef = root.dyn_import(TypeId::of::<M>()).await?;
        Ok(dyn_ref.cast::<M>().expect(ROOT_MODULE_ERR))
    }

    impl dyn RootModule {
        pub async fn import<M: LeafModule + ?Sized>(&'static self)
        -> nxs::Result<&M> {
            import_from(self).await
        }
    }
}

pub mod leaf_module {
    use super::*;
    pub use nxs_interface_macros::LeafModule;

    pub trait LeafModule: DynCast + Sync {
        fn dyn_load(root: &'static dyn RootModule)
        -> BoxFuture<nxs::Result<Box<dyn LeafModule>>>
        where Self: Sized;
    }
}
