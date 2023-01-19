use std::str::FromStr;

use resource_path::ResourcePath;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn convert_path() {
    let raw = "https://api.github.com/a?local=a/b/c";
    let res = ResourcePath::from_str(raw).unwrap();
    let json = serde_json::to_string(&res).unwrap();
    let path = serde_json::from_str::<ResourcePath>(&json).unwrap();
    assert_eq!(res, path)
}
