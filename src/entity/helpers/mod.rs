use anyhow::Context;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn children_of(
    db: &DatabaseConnection,
    parent_id: i32,
) -> anyhow::Result<Vec<crate::entity::people::Model>> {
    use crate::entity::{people, person_parent};
    // First get links to child ids, then fetch people by id to avoid ambiguous SQL when joining
    let links = person_parent::Entity::find()
        .filter(person_parent::Column::ParentId.eq(parent_id))
        .all(db)
        .await?;

    let mut kids = Vec::new();
    for l in links {
        if let Some(m) = people::Entity::find_by_id(l.child_id).one(db).await? {
            kids.push(m);
        }
    }

    Ok(kids)
}

pub async fn parents_of(
    db: &DatabaseConnection,
    child_id: i32,
) -> anyhow::Result<Vec<crate::entity::people::Model>> {
    use crate::entity::{people, person_parent};
    // First get links to parent ids, then fetch people by id to avoid ambiguous SQL when joining
    let links = person_parent::Entity::find()
        .filter(person_parent::Column::ChildId.eq(child_id))
        .all(db)
        .await?;

    let mut parents = Vec::new();
    for l in links {
        if let Some(m) = people::Entity::find_by_id(l.parent_id).one(db).await? {
            parents.push(m);
        }
    }

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

pub async fn create_star_chart(db: &DatabaseConnection, person_id: i32) -> anyhow::Result<()> {
    use crate::entity::star_charts;

    let link = star_charts::ActiveModel {
        person_id: Set(person_id),
        chart_type: Set(String::new()),
        chart_key: Set(String::new()),
        star_count: Set(0),
        star_total: Set(0),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    link.insert(db)
        .await
        .context("failed to insert star chart link")?;
    Ok(())
}
