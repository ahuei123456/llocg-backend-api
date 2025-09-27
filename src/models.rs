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
    pub hearts: HashMap<HeartColor, i64>,
    pub blade_heart: Option<BladeHeartColor>,
}

/// Represents the specific data for creating a Live card.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLiveCard {
    pub score: i64,
    pub hearts: HashMap<HeartColor, i64>,
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
            #[serde(default)]
            groups: Vec<String>,
            #[serde(default)]
            units: Vec<String>,
            #[serde(default)]
            skills: Vec<String>,
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
                "field `card_identifier` must be in the format 'series-set-number-rarity'.",
            ));
        }
        let rarity_code = parts[0].to_string();
        let base_identifier = parts[1];

        let base_parts: Vec<&str> = base_identifier.splitn(3, '-').collect();
        if base_parts.len() != 3 {
            return Err(serde::de::Error::custom(
                "field `card_identifier` must be in the format 'series-set-number-rarity'.",
            ));
        }
        let series_code = base_parts[0].to_string();
        let set_code = base_parts[1].to_string();
        let number_in_set = base_parts[2].to_string();

        match (helper.card_type, &helper.type_specifics) {
            (CardType::Character, Some(CreateCardTypeSpecifics::Character(c)))
                if !c.hearts.is_empty() =>
            {
                Ok(CreateCard {
                    name: helper.name,
                    card_type: helper.card_type,
                    groups: helper.groups,
                    units: helper.units,
                    skills: helper.skills,
                    image_url: helper.image_url,
                    type_specifics: helper.type_specifics,
                    // Add the parsed values
                    series_code,
                    set_code,
                    number_in_set,
                    rarity_code,
                })
            }
            (CardType::Live, Some(CreateCardTypeSpecifics::Live(l))) if !l.hearts.is_empty() => {
                Ok(CreateCard {
                    name: helper.name,
                    card_type: helper.card_type,
                    groups: helper.groups,
                    units: helper.units,
                    skills: helper.skills,
                    image_url: helper.image_url,
                    type_specifics: helper.type_specifics,
                    // Add the parsed values
                    series_code,
                    set_code,
                    number_in_set,
                    rarity_code,
                })
            }
            (CardType::Energy, None) => Ok(CreateCard {
                name: helper.name,
                card_type: helper.card_type,
                groups: helper.groups,
                units: helper.units,
                skills: helper.skills,
                image_url: helper.image_url,
                type_specifics: helper.type_specifics,
                series_code,
                set_code,
                number_in_set,
                rarity_code,
            }),
            (CardType::Character, _) | (CardType::Live, _) => Err(serde::de::Error::custom(
                "`hearts` field is required and must not be empty for Character and Live cards.",
            )),
            _ => Err(serde::de::Error::custom(
                "Mismatch between `card_type` and the data provided in `type_specifics`.",
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

/// Represents the payload for creating a new name variant.
#[derive(Debug, Deserialize)]
pub struct CreateNameVariant {
    pub variant_name: String,
    pub canonical_name: String,
}

/// Represents the payload for creating a new group name variant.
#[derive(Debug, Deserialize)]
pub struct CreateGroupVariant {
    pub variant_name: String,
    pub canonical_name: String,
}

#[cfg(test)]
mod test_character {
    use super::*;

    #[test]
    fn test_create_character_card_deserialization_success() {
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
        assert_eq!(card.series_code, "PL!SP");
        assert_eq!(card.set_code, "bp1");
        assert_eq!(card.number_in_set, "001");
        assert_eq!(card.rarity_code, "R");
        assert_eq!(card.name, "Shibuya Kanon");
        assert_eq!(card.card_type, CardType::Character);
        assert_eq!(card.groups, vec!["Love Live! Superstar!!"]);
        assert_eq!(card.units, vec!["CatChu!"]);
        assert_eq!(
            card.skills,
            vec!["常時 自分のステージにほかのメンバーがいない場合、自分はライブできない。"]
        );
        assert_eq!(
            card.image_url.as_deref(),
            Some(
                "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-001-R.png"
            )
        );
        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Character(c)) => {
                assert_eq!(c.cost, 9);
                assert_eq!(c.blades, 3);
                assert_eq!(c.blade_heart, None);
                assert_eq!(c.hearts.get(&HeartColor::Red), Some(&1));
                assert_eq!(c.hearts.get(&HeartColor::Yellow), Some(&1));
                assert_eq!(c.hearts.get(&HeartColor::Purple), Some(&3));
                assert_eq!(c.hearts.get(&HeartColor::Pink), None);
                assert_eq!(c.hearts.get(&HeartColor::Green), None);
                assert_eq!(c.hearts.get(&HeartColor::Blue), None);
                assert_eq!(c.hearts.get(&HeartColor::Gray), None);
            }
            _ => panic!("Expected Character type specifics"),
        }
    }

    #[test]
    fn test_create_character_card_deserialization_no_skills() {
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

        assert_eq!(
            card.image_url.as_deref(),
            Some(
                "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!SP-bp1-013-N.png"
            )
        );
        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Character(c)) => {
                assert_eq!(c.cost, 9);
                assert_eq!(c.blades, 3);
                assert_eq!(c.blade_heart, None);
                assert_eq!(c.hearts.get(&HeartColor::Red), Some(&1));
                assert_eq!(c.hearts.get(&HeartColor::Yellow), Some(&2));
                assert_eq!(c.hearts.get(&HeartColor::Purple), Some(&1));
                assert_eq!(c.hearts.get(&HeartColor::Pink), None);
                assert_eq!(c.hearts.get(&HeartColor::Green), None);
                assert_eq!(c.hearts.get(&HeartColor::Blue), None);
                assert_eq!(c.hearts.get(&HeartColor::Gray), None);
            }
            _ => panic!("Expected Character type specifics"),
        }
    }

    #[test]
    fn test_create_character_card_deserialization_failure_mismatch() {
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
        assert!(create_card_result.unwrap_err().to_string().contains(
            "`hearts` field is required and must not be empty for Character and Live cards."
        ));
    }

    #[test]
    fn test_create_character_card_deserialization_failure_missing_identifier() {
        let json_payload = r#"
    {
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
        assert!(create_card_result.is_err());
        assert!(
            create_card_result
                .unwrap_err()
                .to_string()
                .contains("missing field `card_identifier`")
        );
    }

    #[test]
    fn test_create_character_card_deserialization_failure_bad_identifier() {
        let json_payload = r#"
    {
        "card_identifier": "PL!SP/bp1/013/N",
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
        assert!(create_card_result.is_err());
        assert!(
            create_card_result.unwrap_err().to_string().contains(
                "field `card_identifier` must be in the format 'series-set-number-rarity'"
            )
        );
    }

    #[test]
    fn test_create_character_card_deserialization_failure_missing_name() {
        let json_payload = r#"
    {
        "card_identifier": "PL!SP-bp1-001-R",
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
        assert!(create_card_result.is_err());
        assert!(
            create_card_result
                .unwrap_err()
                .to_string()
                .contains("missing field `name`")
        );
    }

    #[test]
    fn test_create_character_card_deserialization_failure_missing_card_type() {
        let json_payload = r#"
    {
        "card_identifier": "PL!SP-bp1-001-R",
        "name": "Shibuya Kanon",
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
        assert!(create_card_result.is_err());
        assert!(
            create_card_result
                .unwrap_err()
                .to_string()
                .contains("missing field `card_type`")
        );
    }
}

#[cfg(test)]
mod test_live {
    use super::*;

    #[test]
    fn test_create_live_card_deserialization_failure_missing_groups_ok() {
        let json_payload = r#"
    {
        "card_identifier": "LL-PR-004-PR",
        "name": "Ai♡Scream!",
        "card_type": "Live",
        "units": ["AiScReam"],
        "skills": ["ライブ開始時 相手に何が好き？と聞く。 回答がチョコミントかストロベリーフレイバーかクッキー＆クリームの場合、自分と相手は手札を1枚控え室に置く。 回答があなたの場合、自分と相手はカードを1枚引く。 回答がそれ以外の場合、ライブ終了時まで、自分と相手のステージにいるメンバーは ブレード を得る。"],
        "hearts": { "Red": 3, "Pink": 3, "Green": 3},
        "image_url": "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/PR/LL-PR-004-PR.png",
        "score": 3,
        "special_heart": "Score",
        "blade_heart": "All"
    }
    "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(
            create_card_result.is_ok(),
            "Deserialization failed: {:?}",
            create_card_result.err()
        );

        let card = create_card_result.unwrap();
        assert_eq!(card.series_code, "LL");
        assert_eq!(card.set_code, "PR");
        assert_eq!(card.number_in_set, "004");
        assert_eq!(card.rarity_code, "PR");
        assert_eq!(card.card_type, CardType::Live);
        assert!(card.groups.is_empty());
        assert_eq!(card.units, vec!["AiScReam"]);
        assert_eq!(
            card.skills,
            vec![
                "ライブ開始時 相手に何が好き？と聞く。 回答がチョコミントかストロベリーフレイバーかクッキー＆クリームの場合、自分と相手は手札を1枚控え室に置く。 回答があなたの場合、自分と相手はカードを1枚引く。 回答がそれ以外の場合、ライブ終了時まで、自分と相手のステージにいるメンバーは ブレード を得る。"
            ]
        );

        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Live(l)) => {
                assert_eq!(l.score, 3);
                assert_eq!(l.blade_heart, Some(BladeHeartColor::All));
                assert_eq!(l.special_heart, Some(SpecialHeart::Score));
                assert_eq!(l.hearts.get(&HeartColor::Red), Some(&3));
                assert_eq!(l.hearts.get(&HeartColor::Yellow), None);
                assert_eq!(l.hearts.get(&HeartColor::Purple), None);
                assert_eq!(l.hearts.get(&HeartColor::Gray), None);
                assert_eq!(l.hearts.get(&HeartColor::Pink), Some(&3));
                assert_eq!(l.hearts.get(&HeartColor::Green), Some(&3));
                assert_eq!(l.hearts.get(&HeartColor::Blue), None);
            }
            _ => panic!("Expected Live type specifics"),
        };
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
        assert_eq!(card.groups, vec!["Love Live! Superstar!!"]);
        assert!(card.units.is_empty());
        assert!(card.skills.is_empty());

        match card.type_specifics {
            Some(CreateCardTypeSpecifics::Live(l)) => {
                assert_eq!(l.score, 1);
                assert_eq!(l.blade_heart, None);
                assert_eq!(l.special_heart, Some(SpecialHeart::Score));
                assert_eq!(l.hearts.get(&HeartColor::Red), Some(&1));
                assert_eq!(l.hearts.get(&HeartColor::Yellow), Some(&1));
                assert_eq!(l.hearts.get(&HeartColor::Purple), Some(&1));
                assert_eq!(l.hearts.get(&HeartColor::Gray), Some(&1));
                assert_eq!(l.hearts.get(&HeartColor::Pink), None);
                assert_eq!(l.hearts.get(&HeartColor::Green), None);
                assert_eq!(l.hearts.get(&HeartColor::Blue), None);
            }
            _ => panic!("Expected Live type specifics"),
        };
        assert!(card.units.is_empty());
        assert!(card.skills.is_empty());
    }
}

#[cfg(test)]
mod test_energy {
    use super::*;

    #[test]
    fn test_create_energy_card_deserialization_success() {
        let json_payload = r#"
    {
        "card_identifier": "PL!HS-bp1-031-PE＋",
        "name": "ANYOJI HIME",
        "card_type": "Energy",
        "groups": ["Hasu no Sora Jogakuin School Idol Club"],
        "image_url": "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!HS-bp1-031-PE2.png"
    }
    "#;

        let create_card_result = serde_json::from_str::<CreateCard>(json_payload);
        assert!(
            create_card_result.is_ok(),
            "Deserialization failed: {:?}",
            create_card_result.err()
        );

        let card = create_card_result.unwrap();
        assert_eq!(card.series_code, "PL!HS");
        assert_eq!(card.set_code, "bp1");
        assert_eq!(card.number_in_set, "031");
        assert_eq!(card.rarity_code, "PE＋");
        assert_eq!(card.name, "ANYOJI HIME");
        assert!(card.units.is_empty());
        assert_eq!(card.card_type, CardType::Energy);
        assert_eq!(card.groups, vec!["Hasu no Sora Jogakuin School Idol Club"]);
        assert_eq!(
            card.image_url.as_deref(),
            Some(
                "https://llofficial-cardgame.com/wordpress/wp-content/images/cardlist/BP01/PL!HS-bp1-031-PE2.png"
            )
        );
        assert!(card.skills.is_empty());
        assert!(card.type_specifics.is_none());
    }
}
