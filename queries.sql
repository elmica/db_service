-- Application Queries
-- ===================

-- Get employees: id, firstname, lastname, phonenumber, emailaddress
SELECT
    [EmployeeID],
    [FirstName],
    [LastName],
    [PhoneNumber],
    [EmailAddress]
FROM [EmployeeFiles]
ORDER BY [LastName], [FirstName];

-- Get menu items: id, category, item name
SELECT
    mi.[MenuItemID],
    mc.[MenuCategoryText] AS [Category],
    mi.[MenuItemText] AS [ItemName]
FROM [MenuItems] mi
INNER JOIN [MenuCategories] mc ON mi.[MenuCategoryID] = mc.[MenuCategoryID]
ORDER BY mc.[MenuCategoryText], mi.[MenuItemText];

-- Get menu modifiers: id, name, additional cost
SELECT
    [MenuModifierID],
    [MenuModifierText] AS [Name],
    [AdditionalCost]
FROM [MenuModifiers]
ORDER BY [MenuModifierText];
