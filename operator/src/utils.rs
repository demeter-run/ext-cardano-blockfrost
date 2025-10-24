use argon2::Argon2;
use base64::{engine::general_purpose, Engine};
use bech32::ToBase32;
use kube::{
    api::{Patch, PatchParams},
    core::DynamicObject,
    discovery::ApiResource,
    Api, Client, ResourceExt,
};
use serde_json::json;

use crate::{get_config, BlockfrostPort, Error};

pub async fn patch_resource_status(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let status = json!({ "status": payload });
    let patch_params = PatchParams::default();
    api.patch_status(name, &patch_params, &Patch::Merge(status))
        .await?;
    Ok(())
}

pub fn build_hostname(key: &str) -> (String, String) {
    let config = get_config();
    let extension_subdomain = &config.extension_subdomain;
    let dns_zone = &config.dns_zone;
    let hostname = format!("{extension_subdomain}.{dns_zone}");
    let hostname_key = format!("{key}.{extension_subdomain}.{dns_zone}");

    (hostname, hostname_key)
}

pub async fn build_api_key(crd: &BlockfrostPort) -> Result<String, Error> {
    let config = get_config();

    let namespace = crd.namespace().unwrap();
    let network = &crd.spec.network;
    let version = crd
        .spec
        .blockfrost_version
        .clone()
        .unwrap_or(config.default_blockfrost_version.to_string());

    let name = format!("blockfrost-auth-{}", &crd.name_any());

    let password = format!("{name}{namespace}").as_bytes().to_vec();

    let config = get_config();
    let salt = config.api_key_salt.as_bytes();

    let mut output = vec![0; 8];

    let argon2 = Argon2::default();
    let _ = argon2.hash_password_into(password.as_slice(), salt, &mut output);

    let base64 = general_purpose::URL_SAFE_NO_PAD.encode(output);
    let with_bech = bech32::encode(
        &format!("dmtr_blockfrost_{version}_{network}_"),
        base64.to_base32(),
        bech32::Variant::Bech32,
    )
    .unwrap();

    Ok(with_bech)
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::BlockfrostPortSpec;

    use super::*;

    fn set_configs() {
        env::set_var("DNS_ZONE", "dns_zone");
        env::set_var("EXTENSION_SUBDOMAIN", "extension_subdomain");
        env::set_var("API_KEY_SALT", "api_key_salt");
        env::set_var("METRICS_DELAY", "100");
        env::set_var("PROMETHEUS_URL", "prometheus_url");
        env::set_var("DCU_PER_REQUEST", "preview=5,preprod=5,mainnet=5");
        env::set_var("DEFAULT_BLOCKFROST_VERSION", "v1");
    }

    #[tokio::test]
    async fn test_build_api_key() {
        set_configs();
        let mut crd = BlockfrostPort::new(
            "",
            BlockfrostPortSpec {
                operator_version: "1".to_string(),
                network: "preview".to_string(),
                throughput_tier: "0".to_string(),
                blockfrost_version: Some("v1".to_string()),
                auth_token: None,
            },
        );
        crd.metadata.namespace = Some("namespace".to_string());

        let api_key = build_api_key(&crd).await.unwrap();
        assert!(api_key.starts_with("dmtr_blockfrost_v1_preview_"));
        assert!(api_key.len() <= 63);
    }
    #[tokio::test]
    async fn test_build_hostname() {
        set_configs();
        let key = "dmtr_blockfrost_v1_preview_ashjdcnoasdj";
        let (hostname, hostname_key) = build_hostname(key);

        assert_eq!(hostname, "extension_subdomain.dns_zone".to_string());
        assert_eq!(
            hostname_key,
            "dmtr_blockfrost_v1_preview_ashjdcnoasdj.extension_subdomain.dns_zone".to_string()
        );
    }
}
