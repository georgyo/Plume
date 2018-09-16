-- Your SQL goes here
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    creation_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    kind VARCHAR NOT NULL DEFAULT 'unknown',
    object_id INTEGER NOT NULL DEFAULT 0
)
