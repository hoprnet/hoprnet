use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Account::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Account::ChainKey).string_len(40).not_null())
                    .col(ColumnDef::new(Account::PacketKey).string_len(64).not_null())
                    .col(ColumnDef::new(Account::IsSelf).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_account_chain_key")
                    .table(Account::Table)
                    .col(Account::ChainKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_account_packet_key")
                    .table(Account::Table)
                    .col(Account::PacketKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_account_chain_packet_key")
                    .table(Account::Table)
                    .col(Account::ChainKey)
                    .col(Account::PacketKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Announcement::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Announcement::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Announcement::AccountId).integer().not_null())
                    .col(ColumnDef::new(Announcement::Multiaddress).string().not_null())
                    .col(ColumnDef::new(Announcement::AtBlock).integer().unsigned().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_announcement_account")
                            .from(Announcement::Table, Announcement::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_announcement_account_id")
                    .if_not_exists()
                    .table(Announcement::Table)
                    .col(Announcement::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_announcement_account_id_multiaddress")
                    .if_not_exists()
                    .table(Announcement::Table)
                    .col(Announcement::AccountId)
                    .col(Announcement::Multiaddress)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_announcement_account_id_multiaddress")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_announcement_account_id").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Announcement::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_account_chain_packet_key").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_account_chain_key").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_account_packet_key").to_owned())
            .await?;

        manager.drop_table(Table::drop().table(Account::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    PacketKey,
    ChainKey,
    IsSelf
}

#[derive(DeriveIden)]
enum Announcement {
    Table,
    Id,
    AccountId,
    Multiaddress,
    AtBlock,
}
