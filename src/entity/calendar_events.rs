use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "calendar_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub person_id: i32,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entity::people::Entity",
        from = "Column::PersonId",
        to = "crate::entity::people::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    People,
}

impl Related<crate::entity::people::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::People.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
