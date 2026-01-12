use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sbom_package")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sbom_id: Uuid,
    #[sea_orm(primary_key)]
    pub node_id: String,
    pub group: Option<String>,
    pub version: Option<String>,
    pub package_type: Option<PackageType>,
}

/// Type of the components within an SBOM, mostly based on
/// https://cyclonedx.org/docs/1.6/json/#components_items_type
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "package_type")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum PackageType {
    /// A software application
    #[sea_orm(string_value = "application")]
    Application,
    /// A software framework
    #[sea_orm(string_value = "framework")]
    Framework,
    /// A software library
    #[sea_orm(string_value = "library")]
    Library,
    /// A packaging and/or runtime format
    #[sea_orm(string_value = "container")]
    Container,
    /// A runtime environment which interprets or executes software
    #[sea_orm(string_value = "platform")]
    Platform,
    /// A software operating system without regard to deployment model
    #[sea_orm(string_value = "operating-system")]
    OperatingSystem,
    /// A hardware device such as a processor or chip-set
    #[sea_orm(string_value = "device")]
    Device,
    /// A special type of software that operates or controls a particular type of device
    #[sea_orm(string_value = "device-driver")]
    DeviceDriver,
    /// A special type of software that provides low-level control over a device's hardware
    #[sea_orm(string_value = "firmware")]
    Firmware,
    /// A computer file
    #[sea_orm(string_value = "file")]
    File,
    /// A model based on training data that can make predictions or decisions without being explicitly programmed to do so
    #[sea_orm(string_value = "machine-learning-model")]
    MachineLearningModel,
    /// A collection of discrete values that convey information
    #[sea_orm(string_value = "data")]
    Data,
    /// A cryptographic asset including algorithms, protocols, certificates, keys, tokens, and secrets
    #[sea_orm(string_value = "cryptographic-asset")]
    CryptographicAsset,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::sbom_node::Entity")]
    Node,
    #[sea_orm(
        belongs_to = "super::sbom::Entity",
        from = "Column::SbomId",
        to = "super::sbom::Column::SbomId"
    )]
    Sbom,
    #[sea_orm(
        belongs_to = "super::sbom_package_purl_ref::Entity",
        from = "(Column::SbomId, Column::NodeId)",
        to = "(super::sbom_package_purl_ref::Column::SbomId, super::sbom_package_purl_ref::Column::NodeId)"
    )]
    Purl,
    #[sea_orm(
        belongs_to = "super::sbom_package_cpe_ref::Entity",
        from = "(Column::SbomId, Column::NodeId)",
        to = "(super::sbom_package_cpe_ref::Column::SbomId, super::sbom_package_cpe_ref::Column::NodeId)"
    )]
    Cpe,

    #[sea_orm(
        belongs_to = "super::sbom_package_license::Entity",
        from = "(Column::SbomId, Column::NodeId)",
        to = "(super::sbom_package_license::Column::SbomId, super::sbom_package_license::Column::NodeId)"
    )]
    PackageLicense,
}

impl Related<super::sbom_package_license::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageLicense.def()
    }
}

impl Related<super::sbom_node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl Related<super::sbom::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sbom.def()
    }
}

impl Related<super::sbom_package_purl_ref::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Purl.def()
    }
}

impl Related<super::sbom_package_cpe_ref::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cpe.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::str::FromStr;
    use test_log::test;

    #[test]
    fn package_types() {
        use PackageType::*;

        // The standard conversions
        for (s, t) in [
            ("application", Application),
            ("framework", Framework),
            ("library", Library),
            ("container", Container),
            ("platform", Platform),
            ("operating-system", OperatingSystem),
            ("device", Device),
            ("device-driver", DeviceDriver),
            ("firmware", Firmware),
            ("file", File),
            ("machine-learning-model", MachineLearningModel),
            ("data", Data),
            ("cryptographic-asset", CryptographicAsset),
        ] {
            assert_eq!(PackageType::from_str(s), Ok(t));
            assert_eq!(t.to_string(), s);
            assert_eq!(json!(t), json!(s));
        }

        // Error handling
        assert!(PackageType::from_str("missing").is_err());
        assert_eq!(PackageType::from_str("FiLe"), Ok(File));
    }
}
