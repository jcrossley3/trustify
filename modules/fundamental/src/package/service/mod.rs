use crate::{
    package::model::{
        details::{
            package::PackageDetails, package_version::PackageVersionDetails,
            qualified_package::QualifiedPackageDetails,
        },
        summary::{
            package::PackageSummary, qualified_package::QualifiedPackageSummary,
            r#type::TypeSummary,
        },
    },
    Error,
};
use sea_orm::{
    prelude::Uuid, ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait,
};
use sea_query::{Condition, Order};
use trustify_common::{
    db::{
        limiter::LimiterTrait,
        query::{Filtering, Query},
        Database, Transactional,
    },
    model::{Paginated, PaginatedResults},
};
use trustify_entity::{base_purl, qualified_purl, versioned_purl};

pub struct PackageService {
    db: Database,
}

impl PackageService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn purl_types<TX: AsRef<Transactional>>(
        &self,
        tx: TX,
    ) -> Result<Vec<TypeSummary>, Error> {
        #[derive(FromQueryResult)]
        struct Ecosystem {
            r#type: String,
        }

        let connection = self.db.connection(&tx);

        let ecosystems: Vec<_> = base_purl::Entity::find()
            .select_only()
            .column(base_purl::Column::Type)
            .group_by(base_purl::Column::Type)
            .distinct()
            .order_by(base_purl::Column::Type, Order::Asc)
            .into_model::<Ecosystem>()
            .all(&connection)
            .await?
            .into_iter()
            .map(|e| e.r#type)
            .collect();

        TypeSummary::from_names(&ecosystems, &connection).await
    }

    pub async fn packages_for_type<TX: AsRef<Transactional>>(
        &self,
        r#type: &str,
        query: Query,
        paginated: Paginated,
        tx: TX,
    ) -> Result<PaginatedResults<PackageSummary>, Error> {
        let connection = self.db.connection(&tx);

        let limiter = base_purl::Entity::find()
            .filter(base_purl::Column::Type.eq(r#type))
            .filtering(query)?
            .limiting(&connection, paginated.offset, paginated.limit);

        let total = limiter.total().await?;

        Ok(PaginatedResults {
            items: PackageSummary::from_entities(&limiter.fetch().await?, &connection).await?,
            total,
        })
    }

    pub async fn package<TX: AsRef<Transactional>>(
        &self,
        r#type: &str,
        namespace: Option<String>,
        name: &str,
        tx: TX,
    ) -> Result<Option<PackageDetails>, Error> {
        let connection = self.db.connection(&tx);

        let mut query = base_purl::Entity::find()
            .filter(base_purl::Column::Type.eq(r#type))
            .filter(base_purl::Column::Name.eq(name));

        if let Some(ns) = namespace {
            query = query.filter(base_purl::Column::Namespace.eq(ns));
        } else {
            query = query.filter(base_purl::Column::Namespace.is_null());
        }

        if let Some(package) = query.one(&connection).await? {
            Ok(Some(
                PackageDetails::from_entity(&package, &connection).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn package_version<TX: AsRef<Transactional>>(
        &self,
        r#type: &str,
        namespace: Option<String>,
        name: &str,
        version: &str,
        tx: TX,
    ) -> Result<Option<PackageVersionDetails>, Error> {
        let connection = self.db.connection(&tx);

        let mut query = versioned_purl::Entity::find()
            .left_join(base_purl::Entity)
            .filter(base_purl::Column::Type.eq(r#type))
            .filter(base_purl::Column::Name.eq(name))
            .filter(versioned_purl::Column::Version.eq(version));

        if let Some(ns) = namespace {
            query = query.filter(base_purl::Column::Namespace.eq(ns));
        } else {
            query = query.filter(base_purl::Column::Namespace.is_null());
        }

        let package_version = query.one(&connection).await?;

        if let Some(package_version) = package_version {
            Ok(Some(
                PackageVersionDetails::from_entity(None, &package_version, &connection).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn package_by_uuid<TX: AsRef<Transactional>>(
        &self,
        package_version_uuid: &Uuid,
        tx: TX,
    ) -> Result<Option<PackageDetails>, Error> {
        let connection = self.db.connection(&tx);

        if let Some(package) = base_purl::Entity::find_by_id(*package_version_uuid)
            .one(&connection)
            .await?
        {
            Ok(Some(
                PackageDetails::from_entity(&package, &connection).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn package_version_by_uuid<TX: AsRef<Transactional>>(
        &self,
        package_version_uuid: &Uuid,
        tx: TX,
    ) -> Result<Option<PackageVersionDetails>, Error> {
        let connection = self.db.connection(&tx);

        if let Some(package_version) = versioned_purl::Entity::find_by_id(*package_version_uuid)
            .one(&connection)
            .await?
        {
            Ok(Some(
                PackageVersionDetails::from_entity(None, &package_version, &connection).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn qualified_package_by_uuid<TX: AsRef<Transactional>>(
        &self,
        qualified_package_uuid: &Uuid,
        tx: TX,
    ) -> Result<Option<QualifiedPackageDetails>, Error> {
        let connection = self.db.connection(&tx);

        if let Some(qualified_package) = qualified_purl::Entity::find_by_id(*qualified_package_uuid)
            .one(&connection)
            .await?
        {
            Ok(Some(
                QualifiedPackageDetails::from_entity(None, None, &qualified_package, &connection)
                    .await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn packages<TX: AsRef<Transactional>>(
        &self,
        query: Query,
        paginated: Paginated,
        tx: TX,
    ) -> Result<PaginatedResults<PackageSummary>, Error> {
        let connection = self.db.connection(&tx);

        let limiter = base_purl::Entity::find().filtering(query)?.limiting(
            &connection,
            paginated.offset,
            paginated.limit,
        );

        let total = limiter.total().await?;

        Ok(PaginatedResults {
            items: PackageSummary::from_entities(&limiter.fetch().await?, &connection).await?,
            total,
        })
    }

    pub async fn qualified_packages<TX: AsRef<Transactional>>(
        &self,
        query: Query,
        paginated: Paginated,
        tx: TX,
    ) -> Result<PaginatedResults<QualifiedPackageSummary>, Error> {
        let connection = self.db.connection(&tx);

        let limiter = qualified_purl::Entity::find()
            .left_join(versioned_purl::Entity)
            .filter(
                Condition::any().add(
                    versioned_purl::Column::BasePurlId.in_subquery(
                        base_purl::Entity::find()
                            .filtering(query)?
                            .select_only()
                            .column(base_purl::Column::Id)
                            .into_query(),
                    ),
                ),
            )
            .limiting(&connection, paginated.offset, paginated.limit);

        let total = limiter.total().await?;

        Ok(PaginatedResults {
            items: QualifiedPackageSummary::from_entities(&limiter.fetch().await?, &connection)
                .await?,
            total,
        })
    }
}

#[cfg(test)]
mod test;
