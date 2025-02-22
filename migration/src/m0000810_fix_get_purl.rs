use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!("m0000810_fix_get_purl/get_purl.sql"))
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!("m0000740_ensure_get_purl_fns/get_purl.sql"))
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP FUNCTION IF EXISTS encode_uri_component;
"#,
            )
            .await?;

        Ok(())
    }
}
