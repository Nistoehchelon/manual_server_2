ALTER TABLE "shopping_cart_item_data"
ADD CONSTRAINT "shopping_cart_item_data_name_shopping_cart_item_id" UNIQUE ("name", "shopping_cart_item_id"),
DROP CONSTRAINT "shopping_cart_item_data_name_key";