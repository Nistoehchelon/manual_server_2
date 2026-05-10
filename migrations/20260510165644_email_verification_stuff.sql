-- Add migration script here
CREATE TABLE "email_otps" (
  "primary_key" bigserial NOT NULL,
  PRIMARY KEY ("primary_key"),
  "consumed_at" timestamptz NULL,
  "code" text NOT NULL
);

CREATE TABLE "email_otp_pending_users" (
  "primary_key" bigserial NOT NULL,
  PRIMARY KEY ("primary_key"),
  "email_otp_primary_key" bigint NOT NULL,
  "email" text NOT NULL,
  "username" text NOT NULL,
  "password_hash" text NOT NULL,
  FOREIGN KEY ("email_otp_primary_key") REFERENCES "email_otps" ("primary_key") ON DELETE NO ACTION
);
