-- Add migration script here
CREATE SEQUENCE "public".payment_data_primary_key_seq INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 CACHE 1;

CREATE TABLE "public"."invoice_data" (
    "primary_key" bigint DEFAULT nextval('payment_data_primary_key_seq') NOT NULL,
    "invoice_id" text NOT NULL,
    "name" text NOT NULL,
    "value" text NOT NULL,
    CONSTRAINT "payment_data_pkey" PRIMARY KEY ("primary_key")
)
WITH (oids = false);

CREATE UNIQUE INDEX payment_data_name_key ON public.invoice_data USING btree (name);

CREATE UNIQUE INDEX unique_invoice_id_name ON public.invoice_data USING btree (invoice_id, name);


CREATE SEQUENCE "public".payments_primary_key_seq INCREMENT 1 MINVALUE 1 MAXVALUE 9223372036854775807 CACHE 1;

CREATE TABLE "public"."invoices" (
    "primary_key" bigint DEFAULT nextval('payments_primary_key_seq') NOT NULL,
    "id" text NOT NULL,
    "status" text NOT NULL,
    CONSTRAINT "payments_pkey" PRIMARY KEY ("primary_key")
)
WITH (oids = false);

CREATE UNIQUE INDEX payments_id_key ON public.invoices USING btree (id);


ALTER TABLE ONLY "public"."invoice_data" ADD CONSTRAINT "payment_data_payment_id_fkey" FOREIGN KEY (invoice_id) REFERENCES invoices(id);
