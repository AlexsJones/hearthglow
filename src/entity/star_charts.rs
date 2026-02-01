use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "star_charts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub person_id: i32,

    /// e.g., Food/Regular/Cleaning
    pub chart_type: String,

    /// Optional unique-ish key you can use to avoid duplicates, e.g.
    /// "natal" or "transit:2026-01-31T00:00:00Z"
    pub chart_key: String,

    //Number of stars
    pub star_count: i32,
    // Total stars
    pub star_total: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
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
