CREATE TABLE articles (
  id SERIAL PRIMARY KEY,
  headline VARCHAR NOT NULL,
  body TEXT NOT NULL,
  writer_id int NOT NULL,
  publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
  preview TEXT,
  CONSTRAINT fk_writer
    FOREIGN KEY(writer_id)
      REFERENCES writers(id)
      ON DELETE CASCADE
)