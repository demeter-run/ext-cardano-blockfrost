use argon2::Argon2;
use base64::{
    engine::general_purpose::{self},
    Engine as _,
};
use bech32::ToBase32;
use k8s_openapi::{api::core::v1::Secret, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::{
    api::{Patch, PatchParams, PostParams},
    core::ObjectMeta,
    Api, Client, Resource, ResourceExt,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;
use tracing::info;

use crate::{
    create_resource, get_config, get_resource, kong_consumer, patch_resource, BlockfrostPort, Error,
};

pub async fn handle_auth(client: &Client, crd: &BlockfrostPort) -> Result<String, Error> {
    let key = build_api_key(crd).await?;

    handle_auth_secret(client, crd, &key).await?;
    handle_acl_secret(client, crd).await?;
    handle_consumer(client, crd).await?;

    Ok(key)
}

async fn handle_auth_secret(client: &Client, crd: &BlockfrostPort, key: &str) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = build_auth_name(&crd.name_any());
    let secret = build_auth_secret(crd, key);

    let api = Api::<Secret>::namespaced(client.clone(), &namespace);
    let result = api.get_opt(&name).await?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating auth secret");

        let patch_params = PatchParams::default();
        let patch_data = Patch::Merge(secret);

        api.patch(&name, &patch_params, &patch_data).await?;
    } else {
        info!(resource = crd.name_any(), "Creating auth secret");

        let post_params = PostParams::default();

        api.create(&post_params, &secret).await?;
    }

    Ok(())
}

async fn handle_acl_secret(client: &Client, crd: &BlockfrostPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = build_acl_name(&crd.name_any());
    let secret = build_acl_secret(crd);

    let api = Api::<Secret>::namespaced(client.clone(), &namespace);
    let result = api.get_opt(&name).await?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating acl secret");

        let patch_params = PatchParams::default();
        let patch_data = Patch::Merge(secret);

        api.patch(&name, &patch_params, &patch_data).await?;
    } else {
        info!(resource = crd.name_any(), "Creating acl secret");

        let post_params = PostParams::default();

        api.create(&post_params, &secret).await?;
    }

    Ok(())
}

async fn handle_consumer(client: &Client, crd: &BlockfrostPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = build_auth_name(&crd.name_any());

    let kong_consumer = kong_consumer();

    let (metadata, data, raw) = build_consumer(crd)?;

    let result = get_resource(client.clone(), &namespace, &kong_consumer, &name).await?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating consumer");
        patch_resource(client.clone(), &namespace, kong_consumer, &name, raw).await?;
    } else {
        info!(resource = crd.name_any(), "Creating consumer");
        create_resource(client.clone(), &namespace, kong_consumer, metadata, data).await?;
    }

    Ok(())
}

async fn build_api_key(crd: &BlockfrostPort) -> Result<String, Error> {
    let namespace = crd.namespace().unwrap();
    let name = build_auth_name(&crd.name_any());

    let password = format!("{}{}", name, namespace).as_bytes().to_vec();

    let config = get_config();
    let salt = config.api_key_salt.as_bytes();

    let mut output = vec![0; 16];

    let argon2 = Argon2::default();
    let _ = argon2.hash_password_into(password.as_slice(), salt, &mut output);

    let base64 = general_purpose::URL_SAFE_NO_PAD.encode(output);
    let with_bech = bech32::encode(
        "dmtr_blockfrost",
        base64.to_base32(),
        bech32::Variant::Bech32,
    )
    .unwrap();

    Ok(with_bech)
}

fn build_auth_name(name: &str) -> String {
    format!("blockfrost-auth-{name}")
}

fn build_acl_name(name: &str) -> String {
    format!("blockfrost-acl-{name}")
}

fn build_rate_limit_name(tier: &str) -> String {
    format!("rate-limiting-blockfrost-tier-{}", tier)
}

fn build_auth_secret(crd: &BlockfrostPort, api_key: &str) -> Secret {
    let mut string_data = BTreeMap::new();
    string_data.insert("key".into(), api_key.into());

    let mut labels = BTreeMap::new();
    labels.insert("konghq.com/credential".into(), "key-auth".into());

    let metadata = ObjectMeta {
        name: Some(build_auth_name(&crd.name_any())),
        owner_references: Some(vec![OwnerReference {
            api_version: BlockfrostPort::api_version(&()).to_string(),
            kind: BlockfrostPort::kind(&()).to_string(),
            name: crd.name_any(),
            uid: crd.uid().unwrap(),
            ..Default::default()
        }]),
        labels: Some(labels),
        ..Default::default()
    };

    Secret {
        type_: Some(String::from("Opaque")),
        metadata,
        string_data: Some(string_data),
        ..Default::default()
    }
}

fn build_acl_secret(crd: &BlockfrostPort) -> Secret {
    let mut string_data = BTreeMap::new();
    string_data.insert("group".into(), crd.spec.network.to_string());

    let mut labels = BTreeMap::new();
    labels.insert("konghq.com/credential".into(), "acl".into());

    let metadata = ObjectMeta {
        name: Some(build_acl_name(&crd.name_any())),
        owner_references: Some(vec![OwnerReference {
            api_version: BlockfrostPort::api_version(&()).to_string(),
            kind: BlockfrostPort::kind(&()).to_string(),
            name: crd.name_any(),
            uid: crd.uid().unwrap(),
            ..Default::default()
        }]),
        labels: Some(labels),
        ..Default::default()
    };

    Secret {
        type_: Some(String::from("Opaque")),
        metadata,
        string_data: Some(string_data),
        ..Default::default()
    }
}

fn build_consumer(crd: &BlockfrostPort) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let kong_consumer = kong_consumer();
    let config = get_config();

    let name = crd.name_any();
    let namespace = crd.namespace().unwrap();
    let username = format!("{namespace}.{name}");

    let metadata = ObjectMeta::deserialize(&json!({
      "name": build_auth_name(&crd.name_any()),
      "annotations": {
        "kubernetes.io/ingress.class": config.ingress_class,
        "konghq.com/plugins": build_rate_limit_name(&crd.spec.throughput_tier)
      },
      "ownerReferences": [
        {
          "apiVersion": BlockfrostPort::api_version(&()).to_string(), // @TODO: try to grab this from the owner
          "kind": BlockfrostPort::kind(&()).to_string(), // @TODO: try to grab this from the owner
          "name": crd.name_any(),
          "uid": crd.uid()
        }
      ]
    }))?;

    let data = json!({
      "username": username,
      "credentials": [build_auth_name(&crd.name_any()), build_acl_name(&crd.name_any())]
    });

    let raw = json!({
        "apiVersion": kong_consumer.api_version,
        "kind": kong_consumer.kind,
        "metadata": metadata,
        "username": data["username"],
        "credentials": data["credentials"]
    });

    Ok((metadata, data, raw))
}
