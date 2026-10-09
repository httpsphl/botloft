-- A crew's color (spec 15.1): `#RRGGBB` the owner picked, NULL for none.
ALTER TABLE crews ADD COLUMN color TEXT;
