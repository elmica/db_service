import { readFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import MDBReader from "mdb-reader";

const __dirname = dirname(fileURLToPath(import.meta.url));
const mdbPath = join(__dirname, "ChathamSandwich Backup.mdb");
const buffer = readFileSync(mdbPath);
const reader = new MDBReader(buffer);

// Build employee ID -> name lookup
const employees = reader.getTable("EmployeeFiles").getData();
const employeeNames = new Map<number, string>(
  employees.map((e: { EmployeeID: number; FirstName: string; LastName: string }) => [
    e.EmployeeID,
    [e.FirstName, e.LastName].filter(Boolean).join(" ").trim() || null,
  ])
);

const table = reader.getTable("EmployeeTimeCards");
const rows = table.getData();

const rowsWithNames = rows.map(
  (row: Record<string, unknown> & { EmployeeID: number }) => ({
    ...row,
    EmployeeName: employeeNames.get(row.EmployeeID) ?? null,
  })
);

console.log("EmployeeTimeCards (%d rows):\n", rowsWithNames.length);
console.log(JSON.stringify(rowsWithNames, null, 2));
