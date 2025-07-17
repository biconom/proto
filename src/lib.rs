/*pub mod client {
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/client_file_descriptor_set.bin");
    pub mod account {
        include!("generated/mlmbox.client.account.rs");
    }
    pub mod app {
        include!("generated/mlmbox.client.app.rs");
    }
    pub mod asset {
        include!("generated/mlmbox.client.asset.rs");
    }
    pub mod asset_pair {
        include!("generated/mlmbox.client.asset_pair.rs");
    }
    pub mod auth {
        include!("generated/mlmbox.client.auth.rs");
    }
    pub mod binary {
        include!("generated/mlmbox.client.binary.rs");
    }
    pub mod distribution {
        include!("generated/mlmbox.client.distribution.rs");
    }
    pub mod exchanger {
        include!("generated/mlmbox.client.exchanger.rs");
    }
    pub mod finance {
        include!("generated/mlmbox.client.finance.rs");
    }
    pub mod gift_shop {
        include!("generated/mlmbox.client.gift_shop.rs");
    }
    pub mod google_authenticator {
        include!("generated/mlmbox.client.google_authenticator.rs");
    }
    pub mod locale {
        include!("generated/mlmbox.client.locale.rs");
    }
    pub mod matrix {
        include!("generated/mlmbox.client.matrix.rs");
    }
    pub mod network {
        include!("generated/mlmbox.client.network.rs");
    }
    pub mod product {
        include!("generated/mlmbox.client.product.rs");
    }
}*/
pub mod biconom {
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/biconom_file_descriptor.bin");
    pub mod client {
        pub mod auth {
            include!("generated/biconom.client.auth.rs");
        }
        pub mod confirmation {
            include!("generated/biconom.client.confirmation.rs");
        }
        pub mod currency {
            include!("generated/biconom.client.currency.rs");
        }
        pub mod currency_pair {
            include!("generated/biconom.client.currency_pair.rs");
        }
        pub mod google_authenticator {
            include!("generated/biconom.client.google_authenticator.rs");
        }
        pub mod locale {
            include!("generated/biconom.client.locale.rs");
        }
        pub mod referral_link {
            include!("generated/biconom.client.referral_link.rs");
        }
        pub mod session {
            include!("generated/biconom.client.session.rs");
        }
    }
    pub mod types {
        include!("generated/biconom.types.rs");
    }
}
