CREATE TYPE Section AS ENUM ('news', 'opinions', 'features', 'science', 'sports', 'arts', 'humor',);

CREATE TABLE articles (
  id SERIAL PRIMARY KEY,
  headline VARCHAR NOT NULL,
  focus TEXT NOT NULL,
  slug TEXT NOT NULL,
  body TEXT NOT NULL,
  writer_id int NOT NULL,
  section Section NOT NULL,
  publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
  image_url TEXT,
  drive_file_id TEXT,
  featured BOOLEAN NOT NULL DEFAULT FALSE,
  CONSTRAINT fk_writer
    FOREIGN KEY(writer_id)
      REFERENCES writers(id)
      ON DELETE CASCADE
)
