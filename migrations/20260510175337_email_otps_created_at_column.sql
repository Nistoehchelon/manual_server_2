-- Add migration script here
ALTER TABLE "email_otps"
ADD "created_at" timestamptz NOT NULL;
