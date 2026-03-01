-- Rename tables
ALTER TABLE likes_book RENAME TO taste_books;
ALTER TABLE likes_book_tag RENAME TO taste_book_tags;
ALTER TABLE histories_book RENAME TO history_books;
ALTER TABLE notifications_book RENAME TO notification_books;
ALTER TABLE notifications_book_tag RENAME TO notification_book_tags;

-- FCM token cleanup
ALTER TABLE fcm_token RENAME COLUMN udid TO id;
ALTER TABLE fcm_token RENAME COLUMN fcm_token TO token;
ALTER TABLE fcm_token RENAME TO fcm_tokens;
