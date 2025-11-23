use crate::Pool;
use crate::models::{
    BaseCard, Card, CardType, CardTypeSpecifics, CharacterCard, CreateCard, CreateCardTypeSpecifics,
    FullCard, HeartColor, LiveCard, Printing, RarityType,
};
use futures::try_join;
use std::collections::HashMap;

/// Custom error type for database operations to provide more specific feedback.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Unit not found: {0}")]
    UnitNotFound(String),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub type DbResult<T> = Result<T, DbError>;

/// Fetches a single, fully detailed card from the database by its ID.
pub async fn fetch_full_card(pool: &Pool, id: i64) -> Result<FullCard, sqlx::Error> {
    // Query 1: Fetch the raw card data.
    // We use `fetch_one` which returns an error if no row is found, which is what we want.
    let card = sqlx::query_as::<_, Card>("SELECT * FROM cards WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    // We can run the rest of the queries concurrently for better performance.
    let (name, set_name, groups, units, skills, hearts, printings, type_specifics) = try_join!(
        // Query 2: Get the card name.
        fetch_name_for_card(pool, card.name_id),
        // Query 2: Get the set name.
        fetch_set_name(pool, &card.set_code),
        // Query 3: Get associated groups.
        fetch_groups_for_card(pool, id),
        // Query 4: Get associated units.
        fetch_units_for_card(pool, id),
        // Query 5: Get associated skills.
        fetch_skills_for_card(pool, id),
        // Query 5: Get hearts.
        fetch_hearts_for_card(pool, id),
        // Query 6: Get all printings.
        fetch_printings_for_card(pool, id),
        // Query 7: Get type-specific data.
        fetch_type_specifics(pool, id, card.card_type)
    )?;

    // Assemble the final `FullCard` struct.
    Ok(FullCard {
        base: BaseCard {
            id: card.id,
            series_code: card.series_code,
            set_code: card.set_code,
            number_in_set: card.number_in_set,
            name,
            card_type: card.card_type,
        },
        set_name,
        groups,
        units,
        skills,
        hearts,
        printings,
        type_specifics,
    })
}

/// Creates multiple new cards and all their related data within a single database transaction.
pub async fn create_bulk_cards(
    pool: &Pool,
    rarity_cache: &HashMap<String, RarityType>,
    name_variant_cache: &HashMap<String, String>,
    group_variant_cache: &HashMap<String, String>,
    new_cards: Vec<CreateCard>,
) -> DbResult<Vec<FullCard>> {
    let mut tx = pool.begin().await?;
    let mut created_card_ids = Vec::with_capacity(new_cards.len());

    for card in new_cards {
        // We pass the transaction `tx` to `create_full_card_with_tx`.
        let card_id = create_full_card_with_tx(
            &mut tx,
            rarity_cache,
            name_variant_cache,
            group_variant_cache,
            card,
        )
        .await?;
        created_card_ids.push(card_id);
    }

    if let Err(e) = tx.commit().await {
        // The transaction will be rolled back automatically when `tx` is dropped.
        // We log the error here to make debugging easier.
        eprintln!("Failed to commit transaction for bulk card creation: {}", e);
        // Propagate the error.
        return Err(DbError::Sqlx(e));
    }

    // After successfully committing, fetch all the newly created full cards.
    let mut full_cards = Vec::with_capacity(created_card_ids.len());
    for card_id in created_card_ids {
        full_cards.push(fetch_full_card(pool, card_id).await?);
    }

    Ok(full_cards)
}

/// Creates a new card and all its related data within a single database transaction.
pub async fn create_full_card(
    pool: &Pool,
    rarity_cache: &HashMap<String, RarityType>,
    name_variant_cache: &HashMap<String, String>,
    group_variant_cache: &HashMap<String, String>,
    new_card: CreateCard,
) -> DbResult<FullCard> {
    let mut tx = pool.begin().await?;
    let card_id = create_full_card_with_tx(
        &mut tx,
        rarity_cache,
        name_variant_cache,
        group_variant_cache,
        new_card,
    )
    .await?;
    tx.commit().await?;
    fetch_full_card(pool, card_id).await.map_err(DbError::from)
}

/// Helper to create a card within an existing transaction.
async fn create_full_card_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    rarity_cache: &HashMap<String, RarityType>,
    name_variant_cache: &HashMap<String, String>,
    group_variant_cache: &HashMap<String, String>,
    new_card: CreateCard,
) -> DbResult<i64> {
    // 1a. Look up rarity type from the cache.
    let rarity_type = rarity_cache
        .get(&new_card.rarity_code)
        .cloned()
        .unwrap_or(RarityType::Regular);

    // 1b. Normalize the card name using the cache.
    let canonical_name = name_variant_cache
        .get(&new_card.name)
        .cloned()
        .unwrap_or_else(|| new_card.name.clone());

    // 1c. Upsert the canonical name into the `names` table and get its ID.
    sqlx::query("INSERT INTO names (name) VALUES (?) ON CONFLICT(name) DO NOTHING")
        .bind(&canonical_name)
        .execute(&mut **tx)
        .await?;
    let name_id: i64 = sqlx::query_scalar("SELECT id FROM names WHERE name = ?")
        .bind(&canonical_name)
        .fetch_one(&mut **tx)
        .await?;

    // 1d. Insert the base card with the name_id and get its new ID.
    let card_id = sqlx::query(
        "INSERT INTO cards (series_code, set_code, number_in_set, name_id, card_type)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&new_card.series_code)
    .bind(&new_card.set_code)
    .bind(&new_card.number_in_set)
    .bind(name_id)
    .bind(new_card.card_type)
    .execute(&mut **tx)
    .await?
    .last_insert_rowid();

    // 2. Insert card-specific data (Character, Live, etc.).
    if let Some(specifics) = &new_card.type_specifics {
        match specifics {
            CreateCardTypeSpecifics::Character(c) => {
                sqlx::query("INSERT INTO character_cards (card_id, cost, blades, blade_heart) VALUES (?, ?, ?, ?)")
                    .bind(card_id).bind(c.cost).bind(c.blades).bind(c.blade_heart)
                    .execute(&mut **tx).await?;
            }
            CreateCardTypeSpecifics::Live(l) => {
                sqlx::query("INSERT INTO live_cards (card_id, score, blade_heart, special_heart) VALUES (?, ?, ?, ?)")
                    .bind(card_id).bind(l.score).bind(l.blade_heart).bind(l.special_heart)
                    .execute(&mut **tx).await?;
            }
        }
    }

    // 3. Insert the single printing.
    sqlx::query(
        "INSERT INTO printings (card_id, rarity_code, rarity_type, image_url) VALUES (?, ?, ?, ?)",
    )
    .bind(card_id)
    .bind(&new_card.rarity_code)
    .bind(rarity_type)
    .bind(&new_card.image_url)
    .execute(&mut **tx)
    .await?;

    // 4. Insert hearts.
    if let Some(specifics) = &new_card.type_specifics {
        let hearts = match specifics {
            CreateCardTypeSpecifics::Character(c) => &c.hearts,
            CreateCardTypeSpecifics::Live(l) => &l.hearts,
        };

        for (color, count) in hearts {
            sqlx::query("INSERT INTO card_hearts (card_id, color, count) VALUES (?, ?, ?)")
                .bind(card_id)
                .bind(color)
                .bind(*count)
                .execute(&mut **tx)
                .await?;
        }
    }

    // 5. Link groups. This assumes groups already exist.
    for group_name in &new_card.groups {
        // Normalize the group name using the cache.
        let canonical_group_name = group_variant_cache
            .get(group_name)
            .cloned()
            .unwrap_or_else(|| group_name.clone());
        let group_id_result: Result<i64, sqlx::Error> =
            sqlx::query_scalar("SELECT id FROM groups WHERE name = ?")
                .bind(&canonical_group_name)
                .fetch_one(&mut **tx)
                .await;
        let group_id = match group_id_result {
            Ok(id) => id,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbError::GroupNotFound(canonical_group_name));
            }
            Err(e) => return Err(e.into()),
        };
        sqlx::query("INSERT INTO card_groups (card_id, group_id) VALUES (?, ?)")
            .bind(card_id)
            .bind(group_id)
            .execute(&mut **tx)
            .await?;
    }

    // 6. Link units. This assumes units already exist.
    for unit_name in &new_card.units {
        let unit_id_result: Result<i64, sqlx::Error> =
            sqlx::query_scalar("SELECT id FROM units WHERE name = ?")
                .bind(unit_name)
                .fetch_one(&mut **tx)
                .await;
        let unit_id = match unit_id_result {
            Ok(id) => id,
            Err(sqlx::Error::RowNotFound) => return Err(DbError::UnitNotFound(unit_name.clone())),
            Err(e) => return Err(e.into()),
        };
        sqlx::query("INSERT INTO card_units (card_id, unit_id) VALUES (?, ?)")
            .bind(card_id)
            .bind(unit_id)
            .execute(&mut **tx)
            .await?;
    }

    // 7. Link skills. This will create the skill if it doesn't exist.
    for skill_text in &new_card.skills {
        // Insert the skill text if it doesn't exist, then get its ID.
        // `ON CONFLICT(text) DO NOTHING` is safe and handles the case where the skill already exists.
        sqlx::query("INSERT INTO skills (text) VALUES (?) ON CONFLICT(text) DO NOTHING")
            .bind(&skill_text)
            .execute(&mut **tx)
            .await?;

        let skill_id: i64 = sqlx::query_scalar("SELECT id FROM skills WHERE text = ?")
            .bind(skill_text)
            .fetch_one(&mut **tx)
            .await?;

        sqlx::query("INSERT INTO card_skills (card_id, skill_id) VALUES (?, ?)")
            .bind(card_id)
            .bind(skill_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(card_id)
}

/// Helper function to fetch the name of a card from its name_id.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `name_id` - The ID of the name to look up.
async fn fetch_name_for_card(pool: &Pool, name_id: i64) -> Result<String, sqlx::Error> {
    let row: (String,) = sqlx::query_as("SELECT name FROM names WHERE id = ?")
        .bind(name_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Helper function to fetch the name of a set from its code.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `set_code` - The code of the set to look up (e.g., "bp2").
async fn fetch_set_name(pool: &Pool, set_code: &str) -> Result<String, sqlx::Error> {
    let row: (String,) = sqlx::query_as("SELECT name FROM sets WHERE set_code = ?")
        .bind(set_code)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Helper function to fetch all group names associated with a card.
///
/// # Argumentss
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
async fn fetch_groups_for_card(pool: &Pool, card_id: i64) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT g.name FROM groups g
         JOIN card_groups cg ON g.id = cg.group_id
         WHERE cg.card_id = ?",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await
}

/// Helper function to fetch all unit names associated with a card.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
async fn fetch_units_for_card(pool: &Pool, card_id: i64) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT u.name FROM units u
         JOIN card_units cu ON u.id = cu.unit_id
         WHERE cu.card_id = ?",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await
}

/// Helper function to fetch all skill texts associated with a card.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
async fn fetch_skills_for_card(pool: &Pool, card_id: i64) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT s.text FROM skills s
         JOIN card_skills cs ON s.id = cs.skill_id
         WHERE cs.card_id = ?",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await
}

/// Helper function to fetch the heart counts for a card.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
async fn fetch_hearts_for_card(
    pool: &Pool,
    card_id: i64,
) -> Result<HashMap<HeartColor, i64>, sqlx::Error> {
    let hearts = sqlx::query_as::<_, (HeartColor, i64)>(
        "SELECT color, count FROM card_hearts WHERE card_id = ?",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?;
    Ok(hearts.into_iter().collect())
}

/// Helper function to fetch all printings for a card.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
async fn fetch_printings_for_card(pool: &Pool, card_id: i64) -> Result<Vec<Printing>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM printings WHERE card_id = ?")
        .bind(card_id)
        .fetch_all(pool)
        .await
}

/// Helper function to fetch the type-specific data (Character or Live) for a card.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `card_id` - The ID of the card.
/// * `card_type` - The `CardType` enum for the card.
async fn fetch_type_specifics(
    pool: &Pool,
    card_id: i64,
    card_type: CardType,
) -> Result<Option<CardTypeSpecifics>, sqlx::Error> {
    match card_type {
        CardType::Character => {
            sqlx::query_as::<_, CharacterCard>("SELECT * FROM character_cards WHERE card_id = ?")
                .bind(card_id)
                .fetch_optional(pool)
                .await
                .map(|opt| opt.map(CardTypeSpecifics::Character))
        }
        CardType::Live => {
            sqlx::query_as::<_, LiveCard>("SELECT * FROM live_cards WHERE card_id = ?")
                .bind(card_id)
                .fetch_optional(pool)
                .await
                .map(|opt| opt.map(CardTypeSpecifics::Live))
        }
        CardType::Energy => Ok(None),
    }
}

/// Inserts a new rarity mapping into the database.
pub async fn add_rarity(pool: &Pool, code: &str, r_type: RarityType) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO rarities (rarity_code, rarity_type) VALUES (?, ?)")
        .bind(code)
        .bind(r_type)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deletes a rarity mapping from the database.
pub async fn delete_rarity(
    pool: &Pool,
    code: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM rarities WHERE rarity_code = ?")
        .bind(code)
        .execute(pool)
        .await
}

/// Fetches all sets from the database.
pub async fn fetch_all_sets(pool: &Pool) -> Result<Vec<crate::models::SetResponse>, sqlx::Error> {
    sqlx::query_as("SELECT set_code, name FROM sets")
        .fetch_all(pool)
        .await
}

/// Inserts a new set into the database.
pub async fn add_set(pool: &Pool, set_code: &str, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO sets (set_code, name) VALUES (?, ?)")
        .bind(set_code)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deletes a set from the database by its code.
pub async fn delete_set(
    pool: &Pool,
    set_code: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM sets WHERE set_code = ?")
        .bind(set_code)
        .execute(pool)
        .await
}

/// Fetches all groups from the database.
pub async fn fetch_all_groups(pool: &Pool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT name FROM groups")
        .fetch_all(pool)
        .await
}

/// Inserts a new group into the database.
pub async fn add_group(pool: &Pool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO groups (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deletes a group from the database by its name.
pub async fn delete_group(
    pool: &Pool,
    name: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM groups WHERE name = ?")
        .bind(name)
        .execute(pool)
        .await
}

/// Fetches all units from the database.
pub async fn fetch_all_units(pool: &Pool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT name FROM units")
        .fetch_all(pool)
        .await
}

/// Inserts a new unit into the database.
pub async fn add_unit(pool: &Pool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO units (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deletes a unit from the database by its name.
pub async fn delete_unit(
    pool: &Pool,
    name: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM units WHERE name = ?")
        .bind(name)
        .execute(pool)
        .await
}

/// Fetches all distinct canonical card names from the database.
pub async fn fetch_all_card_names(pool: &Pool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT name FROM names")
        .fetch_all(pool)
        .await
}
