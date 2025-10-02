pub mod biconom {
	pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/biconom_file_descriptor.bin");
	pub mod client {
		pub mod account {
			include!("generated/biconom.client.account.rs");
		}
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
		pub mod distributor {
			include!("generated/biconom.client.distributor.rs");
		}
		pub mod google_authenticator {
			include!("generated/biconom.client.google_authenticator.rs");
		}
		pub mod mnemonic {
			include!("generated/biconom.client.mnemonic.rs");
		}
		pub mod locale {
			include!("generated/biconom.client.locale.rs");
		}
		pub mod invite_link {
			include!("generated/biconom.client.invite_link.rs");
		}
		pub mod password_policy {
			include!("generated/biconom.client.password_policy.rs");
		}
		pub mod session {
			include!("generated/biconom.client.session.rs");
		}
		pub mod wallet_type {
			include!("generated/biconom.client.wallet_type.rs");
		}
		pub mod wallet_type_currency {
			include!("generated/biconom.client.wallet_type_currency.rs");
		}
		pub mod payment_network {
			include!("generated/biconom.client.payment_network.rs");
		}
		pub mod payment_network_currency {
			include!("generated/biconom.client.payment_network_currency.rs");
		}
	}
	pub mod types {
		include!("generated/biconom.types.rs");
	}
}
