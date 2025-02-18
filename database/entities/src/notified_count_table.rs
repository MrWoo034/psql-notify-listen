//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "notified_count_table")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub widget_id: i64,
    pub timestamp: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::widget_table::Entity",
        from = "Column::WidgetId",
        to = "super::widget_table::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    WidgetTable,
}

impl Related<super::widget_table::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WidgetTable.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
