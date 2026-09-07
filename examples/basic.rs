use fishfile::{FishDocument, FishValue};

fn main() -> fishfile::Result<()> {
    // ------------------------------------------------ parse example from prompt
    let text = r#"
        system {
            theme: dark
            accent: blue
        }

        appearance {
            animations: true
            transparency: 0.85

            icons {
                style: tontoo
                size: 48
            }
        }
    "#;

    let doc = FishDocument::parse(text)?;
    println!("Parsed system.theme = {:?}", doc.get("system.theme").unwrap());
    println!("Parsed appearance.icons.size = {:?}", doc.get("appearance.icons.size").unwrap());

    // ------------------------------------------------ create programmatically
    let mut doc2 = FishDocument::new();
    doc2.set("system.theme", "light");
    doc2.set("system.accent", "orange");
    doc2.set("appearance.animations", true);
    doc2.set("appearance.transparency", 0.9);
    doc2.set("appearance.icons.style", "tontoo");
    doc2.set("appearance.icons.size", 64);
    doc2.set("app.tags", FishValue::Array(vec!["editor".into(), "utility".into()]));
    doc2.set("app.count", 42);

    println!("\n--- Generated .fico ---\n{}", doc2.to_string());

    // ------------------------------------------------ JSON interop (before edit)
    let json = doc2.to_json_pretty()?;
    println!("\n--- JSON ---\n{}", json);
    let from_json = FishDocument::from_json_str(&json)?;
    assert_eq!(doc2.get("system.theme"), from_json.get("system.theme"));
    assert_eq!(from_json.get("app.count").unwrap().as_i64(), Some(42));

    // ------------------------------------------------ edit
    doc2.set("system.theme", "dark");
    assert_eq!(doc2.get("system.theme").unwrap().as_str(), Some("dark"));
    doc2.remove("app.count");
    assert!(doc2.get("app.count").is_none());

    // ------------------------------------------------ file I/O
    let tmp = std::env::temp_dir().join("example.fico");
    doc2.write_to_file(&tmp)?;
    println!("\nWritten to {:?}", tmp);
    let loaded = FishDocument::from_file(&tmp)?;
    println!("Loaded back, equal: {}", loaded == doc2);
    assert_eq!(loaded, doc2);

    // ------------------------------------------------ serde
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Settings {
        theme: String,
        size: i64,
    }
    let s = Settings {
        theme: "dark".into(),
        size: 48,
    };
    let doc3 = FishDocument::from_serializable(&s)?;
    let decoded: Settings = doc3.deserialize()?;
    assert_eq!(s, decoded);
    println!("\nSerde roundtrip OK: {:?}", decoded);

    // ------------------------------------------------ macros
    let v = fishfile::fish_value!({"hello" => "world", "num" => 123});
    println!("\nMacro fish_value!: {:?}", v);

    println!("\nAll examples passed!");
    Ok(())
}
