use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "people", rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub calendar_color: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "crate::entity::calendar_events::Entity",
        from = "Column::Id",
        to = "crate::entity::calendar_events::Column::PersonId"
    )]
    CalendarEvents,

    #[sea_orm(
        has_many = "crate::entity::star_charts::Entity",
        from = "Column::Id",
        to = "crate::entity::star_charts::Column::PersonId"
    )]
    StarCharts,

    // person_parent relations are represented on the person_parent entity
}

impl ActiveModelBehavior for ActiveModel {}
