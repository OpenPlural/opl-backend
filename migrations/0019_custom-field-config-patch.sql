ALTER TABLE CustomField
    ADD COLUMN Config
        VARCHAR(255)
        DEFAULT NULL
        AFTER DataType;