-- V002__lesson_language.sql — добавляет language колонку в lesson_progress

ALTER TABLE lesson_progress ADD COLUMN language TEXT NOT NULL DEFAULT 'en';