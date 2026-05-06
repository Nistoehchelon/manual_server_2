-- Add migration script here
CREATE TABLE "user_data" (
  "primary_key" bigserial NOT NULL,
  PRIMARY KEY ("primary_key"),
  "user_id" text NOT NULL,
  "data_name" text NOT NULL,
  "data_value" text NOT NULL
);

ALTER TABLE user_data
ADD CONSTRAINT user_data_unique_user_key UNIQUE (user_id, data_name);

INSERT INTO user_data (user_id, data_name, data_value)
SELECT id, 'bio', bio
FROM users;


ALTER TABLE users
DROP COLUMN bio;
