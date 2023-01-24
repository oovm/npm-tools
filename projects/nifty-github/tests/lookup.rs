use nifty_github::lookup_user_by_email;

#[test]
fn lookup_noreply_email_without_token() {
    let user = lookup_user_by_email(
        "17541209+oovm@users.noreply.github.com",
        "{}",
        None,
        false,
    )
    .unwrap()
    .expect("noreply user");
    assert_eq!(user.id, Some(17541209));
    assert_eq!(user.login.as_deref(), Some("oovm"));
}

#[test]
fn lookup_from_author_map_json() {
    let map = r#"{"aster@vers.site":{"id":17541209,"login":"oovm"}}"#;
    let user = lookup_user_by_email("aster@vers.site", map, None, false)
        .unwrap()
        .expect("mapped user");
    assert_eq!(user.login.as_deref(), Some("oovm"));
}
