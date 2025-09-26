use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Enums for type safety, mapping to database CHECK constraints.
// The `sqlx::Type` derive allows sqlx to map these to TEXT columns.

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "PascalCase")]
pub enum CardType {
    Character,
    Live,
    Energy,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "PascalCase")]
pub enum RarityType {
    Regular,
    Parallel,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "PascalCase")]
pub enum HeartColor {
    Pink,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
    Gray,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "PascalCase")]
pub enum BladeHeartColor {
    Pink,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
    All,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "PascalCase")]
pub enum SpecialHeart {
    Draw,
    Score,
}

// Structs mapping directly to database tables.

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Set {
    pub id: i64,
    pub set_code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Unit {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Card {
    pub id: i64,
    pub series_code: String,
    pub set_code: String,
    pub number_in_set: String,
    pub name: String,
    pub card_type: CardType,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Printing {
    pub id: i64,
    pub card_id: i64,
    pub rarity_code: String,
    pub rarity_type: RarityType,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CardHeart {
    pub card_id: i64,
    pub color: HeartColor,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterCard {
    pub card_id: i64,
    pub cost: i64,
    pub blades: i64,
    pub blade_heart: Option<BladeHeartColor>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LiveCard {
    pub card_id: i64,
    pub score: i64,
    pub blade_heart: Option<BladeHeartColor>,
    pub special_heart: Option<SpecialHeart>,
}

// This struct doesn't map to a table but will be used to return
// a fully composed card object in our API responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct FullCard {
    #[serde(flatten)]
    pub base: Card,
    pub set_name: String,
    pub groups: Vec<String>,
    pub units: Vec<String>,
    pub skills: Vec<String>,
    pub hearts: HashMap<HeartColor, i64>,
    pub printings: Vec<Printing>,
    #[serde(flatten)]
    pub type_specifics: Option<CardTypeSpecifics>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CardTypeSpecifics {
    Character(CharacterCard),
    Live(LiveCard),
}

// --- Structs for API Request Payloads (Creation) ---

/// Represents the specific data for creating a Character card.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateCharacterCard {
    pub cost: i64,
    pub blades: i64,
    pub blade_heart: Option<BladeHeartColor>,
}

/// Represents the specific data for creating a Live card.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLiveCard {
    pub score: i64,
    pub blade_heart: Option<BladeHeartColor>,
    pub special_heart: Option<SpecialHeart>,
}

// This enum is used only for deserializing the `CreateCard` payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CreateCardTypeSpecifics {
    Character(CreateCharacterCard),
    Live(CreateLiveCard),
}

// --- Structs for API Request Payloads ---

/// Represents the full payload for creating a new card.
#[derive(Debug, Serialize)]
pub struct CreateCard {
    pub name: String,
    pub card_type: CardType,
    pub groups: Vec<String>, // Vec<String> assumes groups are identified by name
    #[serde(default)]
    pub units: Vec<String>, 
    #[serde(default)] // If 'skills' is missing in JSON, it will default to an empty Vec.
    pub skills: Vec<String>, 
    pub hearts: HashMap<HeartColor, i64>,
    pub image_url: Option<String>,
    #[serde(flatten)]
    pub type_specifics: Option<CreateCardTypeSpecifics>,

    // These fields are populated by the custom deserializer
    #[serde(skip_serializing)]
    pub series_code: String,
    #[serde(skip_serializing)]
    pub set_code: String,
    #[serde(skip_serializing)]
    pub number_in_set: String,
    #[serde(skip_serializing)]
    pub rarity_code: String,
}

// Custom deserialization to validate that card_type matches type_specifics
impl<'de> Deserialize<'de> for CreateCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            card_identifier: String,
            name: String,
            card_type: CardType,
            groups: Vec<String>,
            #[serde(default)]
            units: Vec<String>,
            #[serde(default)]
            skills: Vec<String>,
            hearts: HashMap<HeartColor, i64>,
            image_url: Option<String>,
            #[serde(flatten)]
            type_specifics: Option<CreateCardTypeSpecifics>,
        }

        // Explicitly handle the deserialization of the helper struct to provide better error context.
        let helper = Helper::deserialize(deserializer).map_err(|err| {
            // Prepend a custom message to the original Serde error.
            serde::de::Error::custom(format!("Failed to parse request payload. Error: {}", err))
        })?;

        // Parse the card_identifier into its components.
        // Example: "PL!S-bp2-001-R" -> ("PL!S", "bp2", "001", "R")
        let parts: Vec<&str> = helper.card_identifier.rsplitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(
                "field `card_identifier` must be in the format 'series-set-number-rarity'",
            ));
        }
        let rarity_code = parts[0].to_string();
        let base_identifier = parts[1];

        let base_parts: Vec<&str> = base_identifier.splitn(3, '-').collect();
        if base_parts.len() != 3 {
            return Err(serde::de::Error::custom("field `card_identifier` must be in the format 'series-set-number-rarity'"));
        }
        let series_code = base_parts[0].to_string();
        let set_code = base_parts[1].to_string();
        let number_in_set = base_parts[2].to_string();

        match (helper.card_type, &helper.type_specifics) {
            (CardType::Character, Some(CreateCardTypeSpecifics::Character(_))) |
            (CardType::Live, Some(CreateCardTypeSpecifics::Live(_))) |
            (CardType::Energy, None) => Ok(CreateCard {
                name: helper.name,
                card_type: helper.card_type,
                groups: helper.groups,
                units: helper.units,
                skills: helper.skills,
                hearts: helper.hearts,
                image_url: helper.image_url,
                type_specifics: helper.type_specifics,
                // Add the parsed values
                series_code,
                set_code,
                number_in_set,
                rarity_code,
            }),
            _ => Err(serde::de::Error::custom(
                "Mismatch between `card_type` and the data provided in `type_specifics`",
            )),
        }
    }
}

/// Represents the payload for creating a new rarity mapping.
#[derive(Debug, Deserialize)]
pub struct CreateRarity {
    pub rarity_code: String,
    pub rarity_type: RarityType,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_card_deserialization_success() {
        let json_payload = r#"
        {
            "card_identifier": "PL!SP-bp1-001-R",
            "name": "Shibuya Kanon",
            "card_type": "Character",
            "groups": ["Love Live! Superstar!!"],
            "units": ["CatChu!"],
            "skills": ["常時 自分のステージにほかのメンバーがいない場合、自分はライブできない。"],
            "hearts": { "Red": 1, "Yellow": 1, "Purple": 3 },
            "image_url": "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-001-R.png",
            "cost": 9,
            "blades": 3
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(create_card_result.is_ok());

        let card = create_card_result.unwrap();
        assert_eq!(card.skills, vec!["常時 自分のステージにほかのメンバーがいない場合、自分はライブできない。"]);
    }

    #[test]
    fn test_create_card_deserialization_no_skills() {
        let json_payload = r#"
        {
            "card_identifier": "PL!SP-bp1-013-N",
            "name": "Tang Keke",
            "card_type": "Character",
            "groups": ["Love Live! Superstar!!"],
            "units": ["KALEIDOSCORE"],
            "hearts": { "Red": 1, "Yellow": 2, "Purple": 1 },
            "image_url": "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-013-N.png",
            "cost": 9,
            "blades": 3
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(create_card_result.is_ok());

        let card = create_card_result.unwrap();
        assert_eq!(card.series_code, "PL!SP");
        assert_eq!(card.set_code, "bp1");
        assert_eq!(card.number_in_set, "013");
        assert_eq!(card.rarity_code, "N");
        assert_eq!(card.name, "Tang Keke");
        assert_eq!(card.card_type, CardType::Character);
        assert_eq!(card.groups, vec!["Love Live! Superstar!!"]);
        assert_eq!(card.units, vec!["KALEIDOSCORE"]);
        assert!(card.skills.is_empty()); // Assert that skills defaulted to an empty Vec
        assert_eq!(card.hearts.get(&HeartColor::Red), Some(&1));
        assert_eq!(card.hearts.get(&HeartColor::Yellow), Some(&2));
        assert_eq!(card.hearts.get(&HeartColor::Purple), Some(&1));
        assert_eq!(card.hearts.get(&HeartColor::Pink), None);
        assert_eq!(card.hearts.get(&HeartColor::Green), None);
        assert_eq!(card.hearts.get(&HeartColor::Blue), None);
        assert_eq!(card.hearts.get(&HeartColor::Gray), None);
        assert_eq!(card.image_url.as_deref(), Some("https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-013-N.png"));
        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Character(c)) => {
                assert_eq!(c.cost, 9);
                assert_eq!(c.blades, 3);
                assert_eq!(c.blade_heart, None);
            }
            _ => panic!("Expected Character type specifics"),
        }
    }

    #[test]
    fn test_create_card_deserialization_failure_mismatch() {
        let json_payload = r#"
        {
            "card_identifier": "PL!S-bp2-001-R",
            "name": "Takami Chika",
            "card_type": "Live",
            "groups": [], "skills": [], "hearts": {},
            "cost": 1, "blades": 1
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(create_card_result.is_err());
        assert!(create_card_result.unwrap_err().to_string().contains("Mismatch between `card_type` and the data provided in `type_specifics`"));
    }

    #[test]
    fn test_create_live_card_deserialization_success() {
        let json_payload = r#"
        {
            "card_identifier": "PL!SP-bp1-023-L",
            "name": "START!! True dreams",
            "card_type": "Live",
            "groups": ["Love Live! Superstar!!"],
            "hearts": { "Red": 1, "Yellow": 1, "Purple": 1, "Gray": 1 },
            "image_url": "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-023-L.png",
            "score": 1,
            "special_heart": "Score"
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(
            create_card_result.is_ok(),
            "Deserialization failed: {:?}",
            create_card_result.err()
        );

        let card = create_card_result.unwrap();
        assert_eq!(card.series_code, "PL!SP");
        assert_eq!(card.set_code, "bp1");
        assert_eq!(card.number_in_set, "023");
        assert_eq!(card.rarity_code, "L");
        assert_eq!(card.card_type, CardType::Live);
        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Live(l)) => {
                assert_eq!(l.score, 1);
                assert_eq!(l.blade_heart, None);
                assert_eq!(l.special_heart, Some(SpecialHeart::Score));
            }
            _ => panic!("Expected Live type specifics"),
        };
        assert!(card.units.is_empty());
        assert!(card.skills.is_empty());
    }

    #[test]
    fn test_create_energy_card_deserialization_success() {
        let json_payload = r#"
        {
            "card_identifier": "PL!S-bp1-101-E",
            "name": "Energy",
            "card_type": "Energy",
            "groups": [],
            "hearts": {},
            "image_url": null
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(
            create_card_result.is_ok(),
            "Deserialization failed: {:?}",
            create_card_result.err()
        );

        let card = create_card_result.unwrap();
        assert_eq!(card.series_code, "PL!S");
        assert_eq!(card.set_code, "bp1");
        assert_eq!(card.number_in_set, "101");
        assert_eq!(card.rarity_code, "E");
        assert!(card.units.is_empty());
        assert_eq!(card.card_type, CardType::Energy); 
        assert!(card.skills.is_empty());
        assert!(card.type_specifics.is_none());
    }

    #[test]
    fn test_create_card_deserialization_invalid_identifier() {
        let json_payload = r#"
        {
            "card_identifier": "PL!S-bp2-001",
            "name": "Takami Chika",
            "card_type": "Character",
            "groups": [], "skills": [], "hearts": {},
            "cost": 1, "blades": 1
        }
        "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(create_card_result.is_err());
        assert!(create_card_result
            .unwrap_err()
            .to_string()
            .contains("field `card_identifier` must be in the format 'series-set-number-rarity'"));
    }
}