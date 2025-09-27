use crate::models::{
    Card, CardType, CardTypeSpecifics, CharacterCard, CreateCard, CreateCardTypeSpecifics,
    FullCard, HeartColor, LiveCard, RarityType,
    Printing,
};
use crate::Pool;
use futures::try_join;
use std::collections::HashMap;

/// Fetches a single, fully detailed card from the database by its ID.
pub async fn fetch_full_card(pool: &Pool, id: i64) -> Result<FullCard, sqlx::Error> {
    // Query 1: Fetch the base card data.
    // We use `fetch_one` which returns an error if no row is found, which is what we want.
    let base_card = sqlx::query_as::<_, Card>("SELECT * FROM cards WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    // We can run the rest of the queries concurrently for better performance.
    let (set_name, groups, units, skills, hearts, printings, type_specifics) = try_join!(
        // Query 2: Get the set name.
        fetch_set_name(pool, &base_card.set_code),
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
        fetch_type_specifics(pool, id, base_card.card_type)
    )?;

    // Assemble the final `FullCard` struct.
    Ok(FullCard {
        base: base_card,
        set_name,
        groups,
        units,
        skills,
        hearts,
        printings,
        type_specifics,
    })
}

/// Creates a new card and all its related data within a single database transaction.
pub async fn create_full_card(
    pool: &Pool,
    rarity_cache: &HashMap<String, RarityType>,
    name_variant_cache: &HashMap<String, String>,
    group_variant_cache: &HashMap<String, String>,
    new_card: CreateCard,
) -> Result<FullCard, sqlx::Error> {
    // Start a transaction. All operations within this block are atomic.
    let mut tx = pool.begin().await?;

    // 1a. Look up rarity type from the cache. Default to 'Regular' if not found.
    let rarity_type = rarity_cache
        .get(&new_card.rarity_code)
        .cloned()
        .unwrap_or(RarityType::Regular);

    // 1b. Normalize the card name using the cache.
    let canonical_name = name_variant_cache
        .get(&new_card.name)
        .cloned()
        .unwrap_or_else(|| new_card.name.clone());

    // 1. Insert the base card and get its new ID.
    let card_id = sqlx::query(
        "INSERT INTO cards (series_code, set_code, number_in_set, name, card_type)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&new_card.series_code)
    .bind(&new_card.set_code)
    .bind(&new_card.number_in_set)
    .bind(&canonical_name)
    .bind(new_card.card_type)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // 2. Insert card-specific data (Character, Live, etc.).
    if let Some(specifics) = &new_card.type_specifics {
        match specifics {
            CreateCardTypeSpecifics::Character(c) => {
                sqlx::query("INSERT INTO character_cards (card_id, cost, blades, blade_heart) VALUES (?, ?, ?, ?)")
                    .bind(card_id).bind(c.cost).bind(c.blades).bind(c.blade_heart)
                    .execute(&mut *tx).await?;
            }
            CreateCardTypeSpecifics::Live(l) => {
                sqlx::query("INSERT INTO live_cards (card_id, score, blade_heart, special_heart) VALUES (?, ?, ?, ?)")
                    .bind(card_id).bind(l.score).bind(l.blade_heart).bind(l.special_heart)
                    .execute(&mut *tx).await?;
            }
        }
    }

    // 3. Insert the single printing.
    sqlx::query("INSERT INTO printings (card_id, rarity_code, rarity_type, image_url) VALUES (?, ?, ?, ?)")
        .bind(card_id)
        .bind(&new_card.rarity_code)
        .bind(rarity_type)
        .bind(&new_card.image_url)
        .execute(&mut *tx).await?;

    // 4. Insert hearts.
    if let Some(specifics) = &new_card.type_specifics {
        let hearts = match specifics {
            CreateCardTypeSpecifics::Character(c) => &c.hearts,
            CreateCardTypeSpecifics::Live(l) => &l.hearts,
        };

        for (color, count) in hearts {
            sqlx::query("INSERT INTO card_hearts (card_id, color, count) VALUES (?, ?, ?)")
                .bind(card_id).bind(color).bind(*count)
                .execute(&mut *tx).await?;
        }
    }

    // 5. Link groups. This assumes groups already exist.
    for group_name in &new_card.groups {
        // Normalize the group name using the cache.
        let canonical_group_name = group_variant_cache
            .get(group_name)
            .cloned()
            .unwrap_or_else(|| group_name.clone());
        let group_id: i64 = sqlx::query_scalar("SELECT id FROM groups WHERE name = ?").bind(canonical_group_name).fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO card_groups (card_id, group_id) VALUES (?, ?)").bind(card_id).bind(group_id).execute(&mut *tx).await?;
    }

    // 6. Link units. This assumes units already exist.
    for unit_name in &new_card.units {
        let unit_id: i64 = sqlx::query_scalar("SELECT id FROM units WHERE name = ?").bind(unit_name).fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO card_units (card_id, unit_id) VALUES (?, ?)").bind(card_id).bind(unit_id).execute(&mut *tx).await?;
    }

    // 7. Link skills. This will create the skill if it doesn't exist.
    for skill_text in &new_card.skills {
        // Insert the skill text if it doesn't exist, then get its ID.
        // `ON CONFLICT(text) DO NOTHING` is safe and handles the case where the skill already exists.
        sqlx::query("INSERT INTO skills (text) VALUES (?) ON CONFLICT(text) DO NOTHING")
            .bind(&skill_text)
            .execute(&mut *tx).await?;

        let skill_id: i64 = sqlx::query_scalar("SELECT id FROM skills WHERE text = ?").bind(skill_text).fetch_one(&mut *tx).await?;

        sqlx::query("INSERT INTO card_skills (card_id, skill_id) VALUES (?, ?)").bind(card_id).bind(skill_id).execute(&mut *tx).await?;
    }

    // If all queries were successful, commit the transaction.
    tx.commit().await?;

    // Fetch and return the newly created card.
    fetch_full_card(pool, card_id).await // This function doesn't need the caches
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
/// # Arguments
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
async fn fetch_printings_for_card(
    pool: &Pool,
    card_id: i64,
) -> Result<Vec<Printing>, sqlx::Error> {
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
        CardType::Character => sqlx::query_as::<_, CharacterCard>("SELECT * FROM character_cards WHERE card_id = ?")
            .bind(card_id).fetch_optional(pool).await.map(|opt| opt.map(CardTypeSpecifics::Character)),
        CardType::Live => sqlx::query_as::<_, LiveCard>("SELECT * FROM live_cards WHERE card_id = ?")
            .bind(card_id).fetch_optional(pool).await.map(|opt| opt.map(CardTypeSpecifics::Live)),
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
pub async fn delete_rarity(pool: &Pool, code: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM rarities WHERE rarity_code = ?")
        .bind(code)
        .execute(pool)
        .await
}
