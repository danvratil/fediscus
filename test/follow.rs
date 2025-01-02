/*
use common::MastodonServer;

#[test]
fn test_follow_from_unknown_user() {
    let mastodon = MastodonServer::new();
    let fediscus = Fediscus::run().unwrap();

    // When a user sends follow request to Fediscus...
    mastodon.follow(mastodon.user1(), fediscusUser).await
        .expect("Failed to send follow request");

    // Fediscus should accept the request
    let activity = mastodon.wait_for_inbox().await
        .expect("Error waiting for incoming activity");
    assert_eq!(activity.r#type, AcctivityType::Accept);
    assert_eq!(activity.actor, fediscusUser);
    assert_eq!(activity.object, mastodon.user1());

    // And follow the user back
    let activity = mastodon.wait_for_inbox().await
        .expect("Error waiting for incoming activity");
    assert_eq!(activity.r#type, AcctivityType::Follow);
    assert_eq!(activity.actor, fediscusUser);
    assert_eq!(activity.object, mastodon.user1());

    // Finally, the Mastodon server should accept the follow request
    mastodon.accept_follow(activity).await
        .expect("Failed to accept follow request");

    // TODO: Test that mastodon.user1() is now being followed.
}

*/