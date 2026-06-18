-- Clean up old data: we're adding the directory_files junction table,
-- but existing files have no directory link. All files will need to be
-- re-added and re-indexed. Other tables are cleaned by cascade and triggers.
DELETE FROM file;

CREATE TABLE IF NOT EXISTS indexed_directory (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    path             TEXT NOT NULL UNIQUE,
    sort_order       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS directory_files (
    directory_id     INTEGER NOT NULL REFERENCES indexed_directory(id) ON DELETE CASCADE,
    file_id          INTEGER NOT NULL REFERENCES file(id) ON DELETE CASCADE,
    PRIMARY KEY (directory_id, file_id)
);
CREATE INDEX idx_directory_files_file_id ON directory_files(file_id);

CREATE TRIGGER IF NOT EXISTS trg_directory_files_ad
AFTER DELETE ON directory_files
BEGIN
    DELETE FROM file WHERE id = OLD.file_id
        AND NOT EXISTS (SELECT 1 FROM directory_files WHERE file_id = OLD.file_id);
END;
