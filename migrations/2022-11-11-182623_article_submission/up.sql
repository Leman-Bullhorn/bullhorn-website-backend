CREATE TABLE article_submission (
  id SERIAL PRIMARY KEY, 
  headline VARCHAR NOT NULL,
  focus TEXT NOT NULL,
  section Section NOT NULL,
  author_id int NOT NULL,
  drive_file_id TEXT NOT NULL,
  thumbnail_url TEXT
)
