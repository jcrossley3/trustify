mod perf;

use lzma::LzmaReader;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use test_context::test_context;
use test_log::test;
use tracing::{info_span, instrument};
use trustify_common::db::test::TrustifyContext;
use trustify_common::db::Transactional;
use trustify_entity::relationship::Relationship;
use trustify_module_fetch::model::sbom::SbomPackage;
use trustify_module_fetch::service::FetchService;
use trustify_module_ingestor::graph::{sbom::spdx::parse_spdx, sbom::spdx::Information, Graph};

#[instrument]
pub fn open_sbom(name: &str) -> anyhow::Result<impl Read> {
    let pwd = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
    let test_data = pwd.join("../etc/test-data");

    let sbom = test_data.join(name);
    Ok(BufReader::new(File::open(sbom)?))
}

#[instrument]
pub fn open_sbom_xz(name: &str) -> anyhow::Result<impl Read> {
    Ok(LzmaReader::new_decompressor(open_sbom(name)?)?)
}

#[test_context(TrustifyContext, skip_teardown)]
#[instrument]
#[test(tokio::test)]
async fn parse_spdx_quarkus(ctx: TrustifyContext) -> Result<(), anyhow::Error> {
    let db = ctx.db;
    let system = Graph::new(db.clone());
    let fetch = FetchService::new(db);

    // nope, has bad license expressions
    let sbom_data = open_sbom("quarkus-bom-2.13.8.Final-redhat-00004.json")?;

    let start = Instant::now();
    let parse_time = start.elapsed();

    let (spdx, _) = info_span!("parse json").in_scope(|| parse_spdx(sbom_data))?;

    let start = Instant::now();
    let tx = system.transaction().await?;

    let sbom = system
        .ingest_sbom(
            "test.com/my-sbom.json",
            "10",
            &spdx.document_creation_information.spdx_document_namespace,
            Information(&spdx),
            &tx,
        )
        .await?;

    sbom.ingest_spdx(spdx, &tx).await?;
    let ingest_time = start.elapsed();
    let start = Instant::now();

    // commit, then test
    tx.commit().await?;

    let described = fetch
        .describes_packages(sbom.sbom.sbom_id, Default::default(), Transactional::None)
        .await?;
    log::info!("{:#?}", described);
    assert_eq!(1, described.items.len());
    let first = &described.items[0];
    assert_eq!(
        &SbomPackage {
            id: "".into(),
            name: "".into(),
            purl: vec![],
            cpe: vec![],
        },
        first
    );

    let contains = fetch
        .related_packages(
            sbom.sbom.sbom_id,
            Relationship::ContainedBy,
            first,
            Transactional::None,
        )
        .await?;

    log::info!("{}", contains.len());

    assert!(contains.len() > 500);

    let query_time = start.elapsed();

    log::info!("parse {}ms", parse_time.as_millis());
    log::info!("ingest {}ms", ingest_time.as_millis());
    log::info!("query {}ms", query_time.as_millis());

    Ok(())
}

#[test_context(TrustifyContext, skip_teardown)]
#[test(tokio::test)]
async fn test_parse_spdx(ctx: TrustifyContext) -> Result<(), anyhow::Error> {
    let db = ctx.db;
    let system = Graph::new(db.clone());
    let fetch = FetchService::new(db);

    let sbom = open_sbom("ubi9-9.2-755.1697625012.json")?;

    let tx = system.transaction().await?;

    let start = Instant::now();
    let (spdx, _) = parse_spdx(sbom)?;
    let parse_time = start.elapsed();

    let start = Instant::now();
    let sbom = system
        .ingest_sbom(
            "test.com/my-sbom.json",
            "10",
            &spdx.document_creation_information.spdx_document_namespace,
            Information(&spdx),
            &tx,
        )
        .await?;

    sbom.ingest_spdx(spdx, &tx).await?;

    tx.commit().await?;

    let ingest_time = start.elapsed();
    let start = Instant::now();

    let described = fetch
        .describes_packages(sbom.sbom.sbom_id, Default::default(), Transactional::None)
        .await?;

    assert_eq!(1, described.total);
    let first = &described.items[0];

    let contains = fetch
        .related_packages(
            sbom.sbom.sbom_id,
            Relationship::ContainedBy,
            first,
            Transactional::None,
        )
        .await?;

    log::info!("{}", contains.len());

    assert!(contains.len() > 500);

    let query_time = start.elapsed();

    log::info!("parse {}ms", parse_time.as_millis());
    log::info!("ingest {}ms", ingest_time.as_millis());
    log::info!("query {}ms", query_time.as_millis());

    Ok(())
}