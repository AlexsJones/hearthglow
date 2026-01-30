use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "person_parent")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub parent_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub child_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::people::Entity",
        from = "Column::ParentId",
        to = "super::people::Column::Id"
    )]
    Parent,

    #[sea_orm(
        belongs_to = "super::people::Entity",
        from = "Column::ChildId",
        to = "super::people::Column::Id"
    )]
    Child,
}

impl ActiveModelBehavior for ActiveModel {}
