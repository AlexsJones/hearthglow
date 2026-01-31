use anyhow::Context;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, Set,
};

pub async fn children_of(
    db: &DatabaseConnection,
    parent_id: i32,
) -> anyhow::Result<Vec<crate::entity::people::Model>> {
    use crate::entity::{people, person_parent};

    let kids = people::Entity::find()
        // start from people (the child rows), join the link rows where people.id = person_parent.child_id
        .join(JoinType::InnerJoin, person_parent::Relation::Child.def())
        .filter(person_parent::Column::ParentId.eq(parent_id))
        .all(db)
        .await?;

    Ok(kids)
}

pub async fn parents_of(
    db: &DatabaseConnection,
    child_id: i32,
) -> anyhow::Result<Vec<crate::entity::people::Model>> {
    use crate::entity::{people, person_parent};

    let parents = people::Entity::find()
        // start from people (the parent rows), join the link rows where people.id = person_parent.parent_id
        .join(JoinType::InnerJoin, person_parent::Relation::Parent.def())
        .filter(person_parent::Column::ChildId.eq(child_id))
        .all(db)
        .await?;

    Ok(parents)
}

pub async fn add_parent_child(
    db: &DatabaseConnection,
    parent_id: i32,
    child_id: i32,
) -> anyhow::Result<()> {
    use crate::entity::person_parent;

    // Optional: prevent parent==child
    anyhow::ensure!(parent_id != child_id, "a person cannot be their own parent");

    let link = person_parent::ActiveModel {
        parent_id: Set(parent_id),
        child_id: Set(child_id),
    };

    link.insert(db)
        .await
        .context("failed to insert parent-child link")?;
    Ok(())
}

pub async fn create_star_chart(
    db: &DatabaseConnection,
    star_chart_id: i32,
    person_id: i32,
) -> anyhow::Result<()> {
    use crate::entity::star_charts;

    let link = star_charts::ActiveModel {
        person_id: Set(person_id),
        chart_type: todo!(),
        chart_key: todo!(),
        data_json: todo!(),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    link.insert(db)
        .await
        .context("failed to insert star chart link")?;
    Ok(())
}
