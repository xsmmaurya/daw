use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DriverEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DriverEvent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            // 🔥 let DB generate UUIDs
                            .default(Expr::cust("gen_random_uuid()"))
                    )
                    .col(
                        ColumnDef::new(DriverEvent::TenantId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(DriverEvent::DriverId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(DriverEvent::ActorUserId)
                            .uuid()
                            .null()
                    )
                    .col(
                        ColumnDef::new(DriverEvent::Kind)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(DriverEvent::Payload)
                            .json_binary()
                            .null()
                    )
                    .col(
                        ColumnDef::new(DriverEvent::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(DriverEvent::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_driver_event_driver_created_at")
                    .table(DriverEvent::Table)
                    .col(DriverEvent::DriverId)
                    .col(DriverEvent::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_driver_event_driver")
                    .from(DriverEvent::Table, DriverEvent::DriverId)
                    .to(Driver::Table, Driver::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_driver_event_tenant")
                    .from(DriverEvent::Table, DriverEvent::TenantId)
                    .to(Tenant::Table, Tenant::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DriverEvent::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum DriverEvent {
    Table,
    Id,
    TenantId,
    DriverId,
    ActorUserId,
    Kind,
    Payload,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Driver {
    Table,
    Id,
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
}
