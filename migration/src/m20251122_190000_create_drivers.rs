use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1) Create drivers table
        manager
            .create_table(
                Table::create()
                    .table(Driver::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Driver::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Driver::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Driver::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Driver::IsOnline)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Driver::Lat)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Driver::Lon)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Driver::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Driver::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_drivers_tenants")
                            .from(Driver::Table, Driver::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_drivers_users")
                            .from(Driver::Table, Driver::UserId)
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
                    .name("idx_drivers_tenant_user")
                    .table(Driver::Table)
                    .col(Driver::TenantId)
                    .col(Driver::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_drivers_tenant_user")
                    .table(Driver::Table)
                    .to_owned(),
            )
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(Driver::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Driver {
    Table,
    Id,
    TenantId,
    UserId,
    IsOnline,
    Lat,
    Lon,
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
