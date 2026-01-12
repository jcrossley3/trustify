use sea_orm_migration::prelude::*;
use sea_query::extension::postgres::Type;
use strum::VariantNames;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the enum
        let builder = manager.get_connection().get_database_backend();
        let values = PackageType::VARIANTS.iter().skip(1).copied();
        let stmt = builder
            .build(Type::create().as_enum(PackageType::Table).values(values))
            .to_string();
        manager.get_connection().execute_unprepared(&stmt).await?;

        // Alter the table to store the enum
        manager
            .alter_table(
                Table::alter()
                    .table(SbomPackage::Table)
                    .add_column(
                        ColumnDef::new(SbomPackage::PackageType)
                            .custom(PackageType::Table)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SbomPackage::Table)
                    .drop_column(SbomPackage::PackageType)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(Type::drop().if_exists().name(PackageType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SbomPackage {
    Table,
    PackageType,
}

#[derive(DeriveIden, strum::VariantNames, strum::Display, Clone)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
#[allow(unused)]
pub enum PackageType {
    Table,
    Application,
    Framework,
    Library,
    Container,
    Platform,
    #[sea_orm(iden = "operating-system")]
    OperatingSystem,
    Device,
    #[sea_orm(iden = "device-driver")]
    DeviceDriver,
    Firmware,
    File,
    #[sea_orm(iden = "machine-learning-model")]
    MachineLearningModel,
    Data,
    #[sea_orm(iden = "cryptographic-asset")]
    CryptographicAsset,
}
