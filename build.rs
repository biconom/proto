fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Compiling proto files...");

    /*tonic_build::configure()
        .out_dir("src/generated")
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path("src/generated/private_file_descriptor_set.bin")
        .compile_protos(&[
            "proto/mlmbox/private/gen_event_img/gen_event_img.proto",
        ], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));*/

    /*tonic_build::configure()
        .out_dir("src/generated")
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .build_server(false)
        .build_client(false)
        .compile_protos(&[
            "proto/mlmbox/local/types/account.proto",
            "proto/mlmbox/local/types/asset.proto",
            "proto/mlmbox/local/types/asset_pair.proto",
            "proto/mlmbox/local/types/distribution.proto",
            "proto/mlmbox/local/types/error.proto",
            "proto/mlmbox/local/types/google_authenticator.proto",
            "proto/mlmbox/local/types/processing.proto",
            "proto/mlmbox/local/types/telegram_bot_events.proto",
            "proto/mlmbox/local/types/uploader.proto",
            "proto/mlmbox/local/types/wallet.proto",

            "proto/mlmbox/local/account/account.proto",
            "proto/mlmbox/local/app/app.proto",
            "proto/mlmbox/local/asset/asset.proto",
            "proto/mlmbox/local/asset/coin_market_cap.proto",
            "proto/mlmbox/local/asset_pair/asset_pair.proto",
            "proto/mlmbox/local/binary/binary.proto",
            "proto/mlmbox/local/distribution/distribution.proto",
            "proto/mlmbox/local/exchanger/exchanger.proto",
            "proto/mlmbox/local/finance/pool_hunter.proto",
            "proto/mlmbox/local/finance/wallet.proto",
            "proto/mlmbox/local/gift_shop/gift_shop.proto",
            "proto/mlmbox/local/google_authenticator/google_authenticator.proto",
            "proto/mlmbox/local/locale/locale.proto",
            "proto/mlmbox/local/matrix/matrix.proto",
            "proto/mlmbox/local/product/product.proto",
        ], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));*/

    tonic_prost_build::configure()
        // .out_dir(std::path::Path::new("src/generated").as_ref())
        .out_dir("src/generated")
        // .type_attribute(".", "#[derive(serde::Serialize)]")
        .build_server(true)
        .build_client(false)
        // .file_descriptor_set_path(std::path::Path::new("src/generated/biconom_file_descriptor.bin").as_ref())
        .file_descriptor_set_path("src/generated/biconom_file_descriptor.bin")
        .compile_protos(&[
                "biconom/types/account.proto",
                "biconom/types/account_policy.proto",
                "biconom/types/bonus.proto",
                "biconom/types/bonus_policy.proto",
                "biconom/types/boundary.proto",
                "biconom/types/calculation.proto",
                "biconom/types/community.proto",
                "biconom/types/community_policy.proto",
                "biconom/types/confirmation.proto",
                "biconom/types/confirmation_policy.proto",
                "biconom/types/currency.proto",
                "biconom/types/currency_policy.proto",
                "biconom/types/currency_pair.proto",
                "biconom/types/currency_pair_policy.proto",
                "biconom/types/distributor.proto",
                "biconom/types/distributor_policy.proto",
                "biconom/types/distributor_branch.proto",
                "biconom/types/distributor_branch_policy.proto",
                "biconom/types/google_authenticator.proto",
                "biconom/types/google_authenticator_policy.proto",
                "biconom/types/locale.proto",
                "biconom/types/mnemonic.proto",
                "biconom/types/network.proto",
                "biconom/types/network_policy.proto",
                "biconom/types/network_account.proto",
                "biconom/types/network_account_policy.proto",
                "biconom/types/network_partition.proto",
                "biconom/types/network_partition_policy.proto",
                "biconom/types/password_policy.proto",
                "biconom/types/presence.proto",
                "biconom/types/invite_link.proto",
                "biconom/types/invite_link_policy.proto",
                "biconom/types/relationship.proto",
                "biconom/types/rounding.proto",
                "biconom/types/session.proto",
                "biconom/types/session_policy.proto",
                "biconom/types/slot.proto",
                "biconom/types/slot_policy.proto",
                "biconom/types/slot_branch.proto",
                "biconom/types/slot_branch_policy.proto",
                "biconom/types/sort.proto",
                "biconom/types/subject.proto",
                "biconom/types/trace.proto",
                "biconom/types/tree.proto",
                "biconom/types/tree_policy.proto",
                "biconom/types/tree_distributor.proto",
                "biconom/types/tree_distributor_policy.proto",
                "biconom/types/tree_partition.proto",
                "biconom/types/tree_partition_policy.proto",
                "biconom/types/user.proto",
                "biconom/types/user_policy.proto",

                "biconom/client/account/account.proto",
                "biconom/client/auth/auth.proto",
                "biconom/client/confirmation/confirmation.proto",
                "biconom/client/currency/currency.proto",
                "biconom/client/currency_pair/currency_pair.proto",
                "biconom/client/distributor/distributor.proto",
                "biconom/client/google_authenticator/google_authenticator.proto",
                "biconom/client/locale/locale.proto",
                "biconom/client/mnemonic/mnemonic.proto",
                "biconom/client/password_policy/password_policy.proto",
                "biconom/client/invite_link/invite_link.proto",
                "biconom/client/session/session.proto",
        ], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));

    println!("Proto files compiled successfully.");
    Ok(())
}
