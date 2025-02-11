use async_trait::async_trait;
use std::sync::atomic::AtomicI64;

use crate::apub;
use crate::db::Uri;
use crate::storage::{
    Account, AccountError, AccountId, AccountStorage, Blog, BlogError, BlogId, BlogStorage, Follow,
    FollowError, FollowId, FollowStorage, Note, NoteError, NoteId, NoteStorage, Storage,
};
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::kinds::actor::PersonType;
use activitypub_federation::protocol::public_key::PublicKey;
use chrono::Utc;
use tokio::sync::Mutex;
use url::Url;

pub fn create_person(name: &str, domain: &str) -> apub::Person {
    apub::Person {
        kind: PersonType::Person,
        id: ObjectId::<Account>::parse(&format!("http://{}/users/{}", domain, name)).unwrap(),
        preferred_username: name.to_string(),
        inbox: Url::parse(&format!("http://{}/inbox", domain)).unwrap(),
        outbox: Some(Url::parse(&format!("http://{}/outbox", domain)).unwrap()),
        shared_inbox: Some(Url::parse(&format!("http://{}/shared_inbox", domain)).unwrap()),
        public_key: PublicKey {
            id: format!("http://{}/users/{}/main-key", domain, name),
            owner: Url::parse(&format!("http://{}/users/{}", domain, name)).unwrap(),
            public_key_pem: "testkey".to_string(),
        },
    }
}

// This is a weak (512 bytes) RSA key intended solely and purely for
// testing  purposes (because generating a proper keypair is too slow
// for tests). Absolutely do not use this anywhere else.
static TEST_USER_RSA_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIBVAIBADANBgkqhkiG9w0BAQEFAASCAT4wggE6AgEAAkEAu/sQ0ol3exwjl3SP
xQVgKqLuVmuCmwnT9XjvePUmyxhxTpIl10ifGpOGKSwAhJaN8bZ/8ar5rV2fFGHn
9dIdewIDAQABAkBdJsDjU7OdkgghGWXVICDTgAHTWVtTML+UOOWSbwlWQmAcKrir
qf1XWcTOIenscuu2ocovL+UdOSWBmqiZeu3BAiEA8VILbAdZRX8Hmg/LN4fmmKa/
lHO6QVkO3DxnvzuFeOMCIQDHamOwNUEqPM2Riv6IvWXkAGnYB5aSEszM6XGs97Gk
iQIgW3ZUG+qhHCTsNaj8xZw9eufgPpyqIDBn4HMIg0NE8C8CIQCQ19PEv0mv1i4y
UtUCmR8S9sHqaciSHRmolL8+tULQkQIgMBpo0gdt9EavSuNyJg25apZAfoswNwmp
56u2h3e3R98=
-----END PRIVATE KEY-----"#;

static TEST_USER_RSA_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBALv7ENKJd3scI5d0j8UFYCqi7lZrgpsJ
0/V473j1JssYcU6SJddInxqThiksAISWjfG2f/Gq+a1dnxRh5/XSHXsCAwEAAQ==
-----END PUBLIC KEY-----"#;

pub fn generate_local_user(host: &str) -> Account {
    Account {
        id: 1.into(),
        uri: Url::parse(&format!("http://{}/users/fediscus", host))
            .unwrap()
            .into(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        username: "fediscus".to_string(),
        host: host.to_string(),
        inbox: Url::parse(&format!("http://{}/users/fediscus/inbox", host))
            .unwrap()
            .into(),
        outbox: Some(
            Url::parse(&format!("http://{}/outbox", host))
                .unwrap()
                .into(),
        ),
        shared_inbox: None,
        public_key: TEST_USER_RSA_PUBLIC_KEY.to_string(),
        private_key: Some(TEST_USER_RSA_PRIVATE_KEY.to_string()),
        local: true,
    }
}

pub struct MemoryStorage {
    accounts: Mutex<Vec<Account>>,
    follows: Mutex<Vec<Follow>>,
    blogs: Mutex<Vec<Blog>>,
    notes: Mutex<Vec<Note>>,

    next_account_id: AtomicI64,
    next_follow_id: AtomicI64,
    next_blog_id: AtomicI64,
    next_note_id: AtomicI64,
}

impl MemoryStorage {
    pub fn new(host: &str) -> Self {
        Self {
            accounts: Mutex::new(vec![generate_local_user(host)]),
            follows: Mutex::new(Vec::new()),
            blogs: Mutex::new(Vec::new()),
            notes: Mutex::new(Vec::new()),

            next_account_id: AtomicI64::new(2),
            next_follow_id: AtomicI64::new(1),
            next_blog_id: AtomicI64::new(1),
            next_note_id: AtomicI64::new(1),
        }
    }
}

#[async_trait]
impl AccountStorage for MemoryStorage {
    async fn new_account(&self, person: &apub::Person) -> Result<Account, AccountError> {
        let mut accounts = self.accounts.lock().await;
        if accounts
            .iter()
            .any(|a| a.uri == person.id.inner().clone().into())
        {
            return Err(AccountError::AlreadyExists);
        }

        let account = Account {
            id: self
                .next_account_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                .into(),
            uri: person.id.inner().clone().into(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            username: person.preferred_username.clone(),
            host: "localhost".to_string(), // Simplified
            inbox: person.inbox.clone().into(),
            outbox: person.outbox.clone().map(Into::into),
            shared_inbox: person.shared_inbox.clone().map(Into::into),
            public_key: person.public_key.public_key_pem.clone(),
            private_key: None,
            local: false,
        };

        accounts.push(account.clone());
        Ok(account)
    }

    async fn account_by_id(&self, id: AccountId) -> Result<Option<Account>, AccountError> {
        Ok(self
            .accounts
            .lock()
            .await
            .iter()
            .find(|a| a.id == id)
            .cloned())
    }

    async fn account_by_uri(&self, uri: &Uri) -> Result<Option<Account>, AccountError> {
        Ok(self
            .accounts
            .lock()
            .await
            .iter()
            .find(|a| a.uri == *uri)
            .cloned())
    }

    async fn get_local_account(&self) -> Result<Account, AccountError> {
        self.accounts
            .lock()
            .await
            .iter()
            .find(|a| a.local)
            .cloned()
            .ok_or(AccountError::NotFound)
    }

    async fn update_or_insert_account(
        &self,
        person: &apub::Person,
    ) -> Result<Account, AccountError> {
        let mut accounts = self.accounts.lock().await;
        let person_uri = person.id.inner().clone().into();
        if let Some(account) = accounts.iter_mut().find(|a| a.uri == person_uri) {
            account.username = person.preferred_username.clone();
            account.inbox = person.inbox.clone().into();
            account.outbox = person.outbox.clone().map(Into::into);
            account.shared_inbox = person.shared_inbox.clone().map(Into::into);
            account.public_key = person.public_key.public_key_pem.clone();
            account.updated_at = Utc::now().naive_utc();
            Ok(account.clone())
        } else {
            let account = Account {
                id: self
                    .next_account_id
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    .into(),
                uri: person.id.inner().clone().into(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                username: person.preferred_username.clone(),
                host: "localhost".to_string(), // Simplified
                inbox: person.inbox.clone().into(),
                outbox: person.outbox.clone().map(Into::into),
                shared_inbox: person.shared_inbox.clone().map(Into::into),
                public_key: person.public_key.public_key_pem.clone(),
                private_key: None,
                local: false,
            };
            accounts.push(account.clone());
            Ok(account)
        }
    }

    async fn delete_account_by_id(&self, id: AccountId) -> Result<(), AccountError> {
        let mut accounts = self.accounts.lock().await;
        if let Some(pos) = accounts.iter().position(|a| a.id == id) {
            accounts.remove(pos);
            Ok(())
        } else {
            Err(AccountError::NotFound)
        }
    }

    async fn delete_account_by_uri(&self, uri: &Uri) -> Result<(), AccountError> {
        let mut accounts = self.accounts.lock().await;
        if let Some(pos) = accounts.iter().position(|a| a.uri == *uri) {
            accounts.remove(pos);
            Ok(())
        } else {
            Err(AccountError::NotFound)
        }
    }
}

#[async_trait]
impl FollowStorage for MemoryStorage {
    async fn new_follow(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
        uri: &Uri,
        pending: bool,
    ) -> Result<Follow, FollowError> {
        let mut follows = self.follows.lock().await;
        if follows.iter().any(|f| f.uri == *uri) {
            return Err(FollowError::AlreadyExists);
        }

        let follow = Follow {
            id: self
                .next_follow_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                .into(),
            created_at: Utc::now().naive_utc(),
            account_id,
            target_account_id,
            uri: uri.clone(),
            pending,
        };

        follows.push(follow.clone());
        Ok(follow)
    }

    async fn follows_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Follow>, FollowError> {
        Ok(self
            .follows
            .lock()
            .await
            .iter()
            .filter(|f| f.account_id == account_id)
            .cloned()
            .collect())
    }

    async fn follow_by_uri(&self, uri: &Uri) -> Result<Option<Follow>, FollowError> {
        Ok(self
            .follows
            .lock()
            .await
            .iter()
            .find(|f| f.uri == *uri)
            .cloned())
    }

    async fn follow_by_ids(
        &self,
        account_id: AccountId,
        target_account_id: AccountId,
    ) -> Result<Option<Follow>, FollowError> {
        Ok(self
            .follows
            .lock()
            .await
            .iter()
            .find(|f| f.account_id == account_id && f.target_account_id == target_account_id)
            .cloned())
    }

    async fn delete_follow_by_uri(&self, uri: &Uri) -> Result<(), FollowError> {
        let mut follows = self.follows.lock().await;
        if let Some(pos) = follows.iter().position(|f| f.uri == *uri) {
            follows.remove(pos);
            Ok(())
        } else {
            Err(FollowError::NotFound)
        }
    }

    async fn delete_follow_by_id(&self, follow_id: FollowId) -> Result<(), FollowError> {
        let mut follows = self.follows.lock().await;
        if let Some(pos) = follows.iter().position(|f| f.id == follow_id) {
            follows.remove(pos);
            Ok(())
        } else {
            Err(FollowError::NotFound)
        }
    }

    async fn follow_accepted(&self, uri: &Uri) -> Result<(), FollowError> {
        let mut follows = self.follows.lock().await;
        if let Some(follow) = follows.iter_mut().find(|f| f.uri == *uri) {
            follow.pending = false;
            Ok(())
        } else {
            Err(FollowError::NotFound)
        }
    }
}

#[async_trait]
impl BlogStorage for MemoryStorage {
    async fn new_blog(&self, url: &Url) -> Result<Blog, BlogError> {
        let mut blogs = self.blogs.lock().await;
        if blogs.iter().any(|b| b.url == *url) {
            return Err(BlogError::AlreadyExists);
        }

        let blog = Blog {
            id: self
                .next_blog_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                .into(),
            created_at: Utc::now().naive_utc(),
            url: url.clone(),
        };
        blogs.push(blog.clone());
        Ok(blog)
    }

    async fn blog_by_id(&self, id: BlogId) -> Result<Option<Blog>, BlogError> {
        Ok(self.blogs.lock().await.iter().find(|b| b.id == id).cloned())
    }

    async fn blog_by_url(&self, url: &Url) -> Result<Option<Blog>, BlogError> {
        Ok(self
            .blogs
            .lock()
            .await
            .iter()
            .find(|b| b.url == *url)
            .cloned())
    }

    async fn delete_blog_by_id(&self, id: BlogId) -> Result<(), BlogError> {
        let mut blogs = self.blogs.lock().await;
        if let Some(pos) = blogs.iter().position(|b| b.id == id) {
            blogs.remove(pos);
            Ok(())
        } else {
            Err(BlogError::NotFound)
        }
    }
}

#[async_trait]
impl NoteStorage for MemoryStorage {
    async fn new_post(
        &self,
        account_id: AccountId,
        uri: Uri,
        reply_to_id: Option<NoteId>,
        root_id: Option<NoteId>,
        blog_id: BlogId,
    ) -> Result<Note, NoteError> {
        let now = Utc::now().naive_utc();
        let mut notes = self.notes.lock().await;
        if notes.iter().any(|n| n.uri == uri) {
            return Err(NoteError::AlreadyExists);
        }
        let note = Note {
            id: self
                .next_note_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                .into(),
            created_at: now,
            updated_at: now,
            account_id,
            uri,
            reply_to_id,
            root_id,
            blog_id,
        };
        notes.push(note.clone());
        Ok(note)
    }

    async fn post_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteError> {
        Ok(self.notes.lock().await.iter().find(|n| n.id == id).cloned())
    }

    async fn post_by_uri(&self, uri: &Uri) -> Result<Option<Note>, NoteError> {
        Ok(self
            .notes
            .lock()
            .await
            .iter()
            .find(|n| n.uri == *uri)
            .cloned())
    }

    async fn delete_post_by_id(&self, id: NoteId) -> Result<(), NoteError> {
        let mut notes = self.notes.lock().await;
        if let Some(pos) = notes.iter().position(|n| n.id == id) {
            notes.remove(pos);
            Ok(())
        } else {
            Err(NoteError::NotFound)
        }
    }

    async fn post_count(&self) -> Result<usize, NoteError> {
        Ok(self.notes.lock().await.len())
    }
}

impl Storage for MemoryStorage {}

#[cfg(test)]
mod test {
    use url::Url;

    use super::*;

    #[tokio::test]
    async fn test_new_account() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");

        let account = storage.new_account(&person).await.unwrap();
        assert_eq!(account.username, "testuser");
    }

    #[tokio::test]
    async fn test_account_by_id() {
        let storage = MemoryStorage::new("example.com");
        let _ = create_person("testuser1", "example.com");
        let person = create_person("testuser2", "example.com");
        let _ = create_person("testuser3", "example.com");

        let account = storage.new_account(&person).await.unwrap();
        let result = storage.account_by_id(account.id).await.unwrap().unwrap();
        assert_eq!(result.id, account.id);
        assert_eq!(result.username, person.preferred_username);
    }

    #[tokio::test]
    async fn test_account_by_uri() {
        let storage = MemoryStorage::new("example.com");
        let _ = create_person("testuser1", "example.com");
        let person = create_person("testuser2", "example.com");
        let _ = create_person("testuser3", "example.com");

        let account = storage.new_account(&person).await.unwrap();
        let result = storage
            .account_by_uri(&person.id.inner().clone().into())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.id, account.id);
        assert_eq!(result.username, person.preferred_username);
    }

    #[tokio::test]
    async fn test_get_local_account() {
        let storage = MemoryStorage::new("example.com");
        let result = storage.get_local_account().await.unwrap();
        assert_eq!(result.id, 1.into());
        assert_eq!(result.username, "fediscus");
    }

    #[tokio::test]
    async fn test_update_or_insert_account() {
        let storage = MemoryStorage::new("exaple.com");
        let mut person = create_person("testuser", "example.com");

        // Insert new account
        let account = storage.update_or_insert_account(&person).await.unwrap();
        assert_eq!(account.username, "testuser");

        // Update existing account
        person.preferred_username = "updateduser".to_string();
        let updated_account = storage.update_or_insert_account(&person).await.unwrap();
        assert_eq!(updated_account.id, account.id);
        assert_eq!(updated_account.username, "updateduser");
    }

    #[tokio::test]
    async fn test_delete_account_by_id() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");

        let account = storage.new_account(&person).await.unwrap();
        storage.delete_account_by_id(account.id).await.unwrap();

        let result = storage.account_by_id(account.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_account_by_uri() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");

        let _ = storage.new_account(&person).await.unwrap();
        storage
            .delete_account_by_uri(&person.id.inner().clone().into())
            .await
            .unwrap();

        let result = storage
            .account_by_uri(&person.id.inner().clone().into())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_new_follow() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        assert_eq!(follow.account_id, account1.id);
        assert_eq!(follow.target_account_id, account2.id);
    }

    #[tokio::test]
    async fn test_follows_by_account_id() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let _ = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        let follows = storage.follows_by_account_id(account1.id).await.unwrap();
        assert_eq!(follows.len(), 1);
        assert_eq!(follows[0].target_account_id, account2.id);
    }

    #[tokio::test]
    async fn test_follow_by_uri() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        let result = storage
            .follow_by_uri(&person2.id.inner().clone().into())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.id, follow.id);
    }

    #[tokio::test]
    async fn test_follow_by_ids() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        let result = storage
            .follow_by_ids(account1.id, account2.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.id, follow.id);
    }

    #[tokio::test]
    async fn test_delete_follow_by_uri() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        storage
            .delete_follow_by_uri(&person2.id.inner().clone().into())
            .await
            .unwrap();

        let result = storage.follow_by_uri(&follow.uri).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_follow_by_id() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        storage.delete_follow_by_id(follow.id).await.unwrap();

        let result = storage
            .follow_by_uri(&person2.id.inner().clone().into())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_follow_accepted() {
        let storage = MemoryStorage::new("example.com");
        let person1 = create_person("testuser1", "example.com");
        let person2 = create_person("testuser2", "example.com");

        let account1 = storage.new_account(&person1).await.unwrap();
        let account2 = storage.new_account(&person2).await.unwrap();

        let follow = storage
            .new_follow(
                account1.id,
                account2.id,
                &person2.id.inner().clone().into(),
                true,
            )
            .await
            .unwrap();
        assert!(follow.pending);
        storage
            .follow_accepted(&person2.id.inner().clone().into())
            .await
            .unwrap();

        let result = storage
            .follow_by_uri(&person2.id.inner().clone().into())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.id, follow.id);
        assert!(!result.pending);
    }

    #[tokio::test]
    async fn test_new_blog() {
        let storage = MemoryStorage::new("example.com");
        let url = Url::parse("https://example.com/blog").unwrap();

        let blog = storage.new_blog(&url).await.unwrap();
        assert_eq!(blog.url, url);
    }

    #[tokio::test]
    async fn test_blog_by_id() {
        let storage = MemoryStorage::new("example.com");
        let url = Url::parse("https://example.com/blog").unwrap();

        let blog = storage.new_blog(&url).await.unwrap();
        let result = storage.blog_by_id(blog.id).await.unwrap().unwrap();
        assert_eq!(result.id, blog.id);
        assert_eq!(result.url, url);
    }

    #[tokio::test]
    async fn test_blog_by_url() {
        let storage = MemoryStorage::new("example.com");
        let url = Url::parse("https://example.com/blog").unwrap();

        let blog = storage.new_blog(&url).await.unwrap();
        let result = storage.blog_by_url(&url).await.unwrap().unwrap();
        assert_eq!(result.id, blog.id);
        assert_eq!(result.url, url);
    }

    #[tokio::test]
    async fn test_delete_blog_by_id() {
        let storage = MemoryStorage::new("example.com");
        let url = Url::parse("https://example.com/blog").unwrap();

        let blog = storage.new_blog(&url).await.unwrap();
        storage.delete_blog_by_id(blog.id).await.unwrap();

        let result = storage.blog_by_id(blog.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_new_post() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");
        let account = storage.new_account(&person).await.unwrap();
        let blog_url = Url::parse("https://example.com/blog").unwrap();
        let blog = storage.new_blog(&blog_url).await.unwrap();
        let uri = Url::parse("https://example.com/note/1").unwrap();

        let note = storage
            .new_post(account.id, uri.into(), None, None, blog.id)
            .await
            .unwrap();
        assert_eq!(note.account_id, account.id);
        assert_eq!(note.blog_id, blog.id);
    }

    #[tokio::test]
    async fn test_post_by_id() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");
        let account = storage.new_account(&person).await.unwrap();
        let blog_url = Url::parse("https://example.com/blog").unwrap();
        let blog = storage.new_blog(&blog_url).await.unwrap();
        let uri = Url::parse("https://example.com/note/1").unwrap();

        let note = storage
            .new_post(account.id, uri.into(), None, None, blog.id)
            .await
            .unwrap();
        let result = storage.post_by_id(note.id).await.unwrap().unwrap();
        assert_eq!(result.id, note.id);
    }

    #[tokio::test]
    async fn test_post_by_uri() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");
        let account = storage.new_account(&person).await.unwrap();
        let blog_url = Url::parse("https://example.com/blog").unwrap();
        let blog = storage.new_blog(&blog_url).await.unwrap();
        let uri = Url::parse("https://example.com/note/1").unwrap();

        let note = storage
            .new_post(account.id, uri.clone().into(), None, None, blog.id)
            .await
            .unwrap();
        let result = storage.post_by_uri(&uri.into()).await.unwrap().unwrap();
        assert_eq!(result.id, note.id);
    }

    #[tokio::test]
    async fn test_delete_post_by_id() {
        let storage = MemoryStorage::new("example.com");
        let person = create_person("testuser", "example.com");
        let account = storage.new_account(&person).await.unwrap();
        let blog_url = Url::parse("https://example.com/blog").unwrap();
        let blog = storage.new_blog(&blog_url).await.unwrap();
        let uri = Url::parse("https://example.com/note/1").unwrap();

        let note = storage
            .new_post(account.id, uri.into(), None, None, blog.id)
            .await
            .unwrap();
        storage.delete_post_by_id(note.id).await.unwrap();

        let result = storage.post_by_id(note.id).await.unwrap();
        assert!(result.is_none());
    }
}
