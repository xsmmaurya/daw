use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RideEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RideEvent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            // 🔥 let DB generate UUIDs
                            .default(Expr::cust("gen_random_uuid()"))
                    )
                    .col(
                        ColumnDef::new(RideEvent::TenantId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(RideEvent::RideId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(RideEvent::ActorUserId)
                            .uuid()
                            .null()
                    )
                    .col(
                        ColumnDef::new(RideEvent::Kind)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(RideEvent::Payload)
                            .json_binary()
                            .null()
                    )
                    .col(
                        ColumnDef::new(RideEvent::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(RideEvent::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .to_owned(),
            )
            .await?;

        // Index: ride + created_at (timeline)
        manager
            .create_index(
                Index::create()
                    .name("idx_ride_event_ride_created_at")
                    .table(RideEvent::Table)
                    .col(RideEvent::RideId)
                    .col(RideEvent::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // FK: ride_event.ride_id -> ride.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_ride_event_ride")
                    .from(RideEvent::Table, RideEvent::RideId)
                    .to(Ride::Table, Ride::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // FK: ride_event.tenant_id -> tenant.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_ride_event_tenant")
                    .from(RideEvent::Table, RideEvent::TenantId)
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
            .drop_table(Table::drop().table(RideEvent::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum RideEvent {
    Table,
    Id,
    TenantId,
    RideId,
    ActorUserId,
    Kind,
    Payload,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Ride {
    Table,
    Id,
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
}
