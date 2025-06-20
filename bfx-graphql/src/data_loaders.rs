#[macro_export]
macro_rules! data_loader {
    ($ty:ident) => {
        use async_graphql::dataloader::DataLoader;

        pub struct $ty {
            ctx: $crate::context::GlobalContext,
        }

        impl $ty {
            #[must_use]
            pub const fn new(ctx: $crate::context::GlobalContext) -> Self {
                Self { ctx }
            }

            #[must_use]
            pub fn data_loader(ctx: $crate::context::GlobalContext) -> DataLoader<Self> {
                DataLoader::new(Self::new(ctx), tokio::spawn)
            }
        }
    };
}
