DROP TRIGGER IF EXISTS trig_delete_folder;

CREATE TRIGGER trig_delete_folder
    BEFORE DELETE
    ON Folder
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    WITH RECURSIVE FolderHierarchy AS (
        SELECT ID, UserId FROM Folder WHERE ID = OLD.ID
        UNION ALL
        SELECT f.ID, f.UserId FROM Folder f INNER JOIN FolderHierarchy fh ON fh.ID = f.ParentId
    )
    SELECT ID, 0, UserId FROM FolderHierarchy;
END;