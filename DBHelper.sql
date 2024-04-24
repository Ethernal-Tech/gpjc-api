IF NOT EXISTS (SELECT name
FROM sys.databases
WHERE name = 'gpjc_data')
BEGIN
    CREATE DATABASE gpjc_data;
END;
GO