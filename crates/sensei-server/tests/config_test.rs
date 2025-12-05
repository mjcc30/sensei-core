use sensei_server::config::load_prompts;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn load_prompts_works() {
    let yaml = r#"
agents:
  router:
    prompt: "You are a Router."
  red_team:
    prompt: "You are Red."
"#;
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", yaml).unwrap();

    let config = load_prompts(file.path().to_str().unwrap()).unwrap();

    assert_eq!(
        config.agents.get("router").unwrap().prompt,
        "You are a Router."
    );
    assert_eq!(
        config.agents.get("red_team").unwrap().prompt,
        "You are Red."
    );
}
