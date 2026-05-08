CREATE TABLE IF NOT EXISTS activity_stream (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL,
    stream_type TEXT NOT NULL,
    data TEXT NOT NULL,
    series_type TEXT NOT NULL,
    original_size INTEGER NOT NULL,
    resolution TEXT NOT NULL,
    FOREIGN KEY(activity_id) REFERENCES activity(id) ON DELETE CASCADE
);

CREATE INDEX idx_activity_stream_activity_id ON activity_stream(activity_id);
CREATE UNIQUE INDEX idx_activity_stream_activity_type ON activity_stream(activity_id, stream_type);
