use fediscus_activitypub::testing::server::{listen, new_instance, DbPost};
use serial_test::serial;
use tracing::info;

mod common;

use common::FediscusServer;
use url::Url;

#[tokio::test]
#[serial]
async fn test_followed_user_posts_note_without_tag_and_url() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // And then creates a new post that doesn't contain link or the #fediscus tag
    let post = DbPost::new(
        "Hello, Fediscus!".to_string(),
        test_server.local_user().ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server
        .local_user()
        .post(post, &test_server.to_request_data())
        .await
        .expect("Failed to post note");

    // No posts should be created on the Fediscus server
    assert_eq!(
        fediscus
            .service
            .storage()
            .post_count()
            .await
            .expect("Failed to count posts"),
        0
    );
}

#[tokio::test]
#[serial]
async fn test_followed_user_posts_note_with_tag_but_no_url() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // And then creates a new post that contains the #fediscus tag but no link
    let post = DbPost::new(
        "Hello, #Fediscus".to_string(),
        test_server.local_user().ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server
        .local_user()
        .post(post, &test_server.to_request_data())
        .await
        .expect("Failed to post note");

    // No posts should be created on the Fediscus server
    assert_eq!(
        fediscus
            .service
            .storage()
            .post_count()
            .await
            .expect("Failed to count posts"),
        0
    );
}

#[tokio::test]
#[serial]
async fn test_followed_user_posts_note_with_url_but_no_tag() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // And then creates a new post that contains a link but no #fediscus tag
    let post = DbPost::new(
        "Hello, https://example.com/blog-post".to_string(),
        test_server.local_user().ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server
        .local_user()
        .post(post, &test_server.to_request_data())
        .await
        .expect("Failed to post note");

    // No posts should be created on the Fediscus server
    assert_eq!(
        fediscus
            .service
            .storage()
            .post_count()
            .await
            .expect("Failed to count posts"),
        0
    );
}

#[tokio::test]
#[serial]
async fn test_followed_user_posts_note_with_url_and_tag() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // And then creates a new post that contains a link and the #fediscus tag
    let post = DbPost::new(
        "My new post! https://example.com/blog-post #fediscus".to_string(),
        test_server.local_user().ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server
        .local_user()
        .post(post, &test_server.to_request_data())
        .await
        .expect("Failed to post note");

    // A post should be created on the Fediscus server
    assert_eq!(
        fediscus
            .service
            .storage()
            .post_count()
            .await
            .expect("Failed to count posts"),
        1
    );
    let url = Url::parse("https://example.com/blog-post").unwrap();
    let post = fediscus
        .service
        .storage()
        .post_by_id(1.into())
        .await
        .expect("Failed to get post")
        .expect("Post not found");
    let blog = fediscus
        .service
        .storage()
        .blog_by_url(&url)
        .await
        .expect("Failed to get blog")
        .expect("Blog not found");
    assert_eq!(post.blog_id, blog.id);
    assert_eq!(post.root_id, None);
    assert_eq!(post.reply_to_id, None);
}

#[tokio::test]
#[serial]
async fn test_followed_user_posts_reply_to_note() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let fediscus = FediscusServer::new()
        .await
        .expect("Failed to start Fediscus server");
    info!("Fediscus server started");

    let test_server = new_instance("localhost:8087", "testuser".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server).expect("Failed to start test server");
    info!("Test server listening");

    let test_server2 = new_instance("localhost:8088", "testuser2".to_string())
        .await
        .expect("Failed to start test server");
    listen(&test_server2).expect("Failed to start test server");
    info!("Test server 2 listening");

    // The test users follows fediscus
    test_server
        .local_user()
        .follow("fediscus@localhost:8086", &test_server.to_request_data())
        .await
        .expect("Failed to follow Fediscus");
    // We need the other user to follow Fediscus as well, otherwise it won't send the reply
    // to the Fediscus server - this is limitation of the test server implementation, in real
    // world we will receive replies from other users even if we don't follow them.
    test_server2
        .local_user()
        .follow("fediscus@localhost:8086", &test_server2.to_request_data())
        .await
        .expect("Failed to follow Fediscus");

    // And then creates a new post that contains a link and the #fediscus tag
    let post = DbPost::new(
        "My new post! https://example.com/blog-post #fediscus".to_string(),
        test_server.local_user().ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server
        .local_user()
        .post(post.clone(), &test_server.to_request_data())
        .await
        .expect("Failed to post note");

    // Another user creates a reply to the note
    let reply = DbPost::new_reply(
        "Wow, this is a great post!".to_string(),
        test_server2.local_user().ap_id.clone(),
        post.ap_id.clone(),
    )
    .expect("Failed to create post");
    test_server2
        .local_user()
        .post(reply, &test_server2.to_request_data())
        .await
        .expect("Failed to post note");

    // Two posts should be created on the Fediscus server
    assert_eq!(
        fediscus
            .service
            .storage()
            .post_count()
            .await
            .expect("Failed to count posts"),
        2
    );
    let url = Url::parse("https://example.com/blog-post").unwrap();
    let post = fediscus
        .service
        .storage()
        .post_by_id(1.into())
        .await
        .expect("Failed to get post")
        .expect("Post not found");
    let reply = fediscus
        .service
        .storage()
        .post_by_id(2.into())
        .await
        .expect("Failed to get post")
        .expect("Post reply not found");
    let blog = fediscus
        .service
        .storage()
        .blog_by_url(&url)
        .await
        .expect("Failed to get blog")
        .expect("Blog not found");
    assert_eq!(post.root_id, None);
    assert_eq!(post.reply_to_id, None);
    assert_eq!(post.blog_id, blog.id);
    assert_eq!(reply.blog_id, blog.id);
    assert_eq!(reply.reply_to_id, Some(post.id));
    assert_eq!(reply.root_id, Some(post.id));
}
