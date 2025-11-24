use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1) Create rides table
        manager
            .create_table(
                Table::create()
                    .table(Ride::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ride::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Ride::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::RiderId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::DriverId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Ride::PickupLat)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::PickupLon)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::PickupAddress)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Ride::DestLat)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::DestLon)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::DestAddress)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Ride::Tier)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::PaymentMethodId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ride::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Ride::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rides_tenants")
                            .from(Ride::Table, Ride::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rides_riders")
                            .from(Ride::Table, Ride::RiderId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rides_drivers")
                            .from(Ride::Table, Ride::DriverId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 2) Create index separately
        manager
            .create_index(
                Index::create()
                    .name("idx_rides_tenant_status_created_at")
                    .table(Ride::Table)
                    .col(Ride::TenantId)
                    .col(Ride::Status)
                    .col(Ride::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_rides_tenant_status_created_at")
                    .table(Ride::Table)
                    .to_owned(),
            )
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(Ride::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Ride {
    Table,
    Id,
    TenantId,
    RiderId,
    DriverId,
    PickupLat,
    PickupLon,
    PickupAddress,
    DestLat,
    DestLon,
    DestAddress,
    Tier,
    PaymentMethodId,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
