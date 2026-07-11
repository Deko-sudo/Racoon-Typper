-- Preserve the language selected for custom text layout checks.
ALTER TABLE custom_texts ADD COLUMN language TEXT NOT NULL DEFAULT 'en';
